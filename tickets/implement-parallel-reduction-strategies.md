---
id: implement-parallel-reduction-strategies
title: Implement parallel reduction strategies
status: todo
priority: p1
dependencies: [implement-first-profile-numerical-policies, implement-boundary-property-enforcers, implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reduction, scheduling, numerics]
---
## User-visible outcome

A reduction can be scheduled as a single-workgroup or multi-pass strategy — not only the serial order — with the numerical legality of each order checked against the declared realization, so larger reductions stop being serialized by default.

Add single-workgroup and multi-pass reductions beyond the serial schedule. Define empty identities, accumulation dtype, deterministic/relaxed orders, synchronization, partial storage, feasibility and numerical evidence; selection may deliberately choose multiple kernels.

## Dependency notes (2026-07-28)

**`implement-boundary-property-enforcers` is `deferred`, and its restart condition is a failing test rather than a person.** `frontier.rs::the_bounded_profile_admits_no_undischarged_boundary` (`crates/tiler-compiler/src/frontier.rs:2107`) asserts that the bounded profile's two constant property sets discharge each other; when it fails, that ticket becomes startable and the mismatch that failed it is the enforcer's first real case. The derivation, the per-dimension table, and the list of changes that would fire it are at `tickets/implement-boundary-property-enforcers.md:23-50`; do not restate them. A multi-pass reduction writes partial results one pass consumes, which is a per-region boundary variation the single-region profile has never produced — so this ticket is a candidate for firing that trigger rather than a consumer of the enforcers.

**`implement-first-profile-numerical-policies` is `in-progress` and its work is uncommitted.** Worktree `.claude/worktrees/agent-ad2893b1fba4d7f5b`, branch `tkt/implement-first-profile-numerical-policies`, tip `06af0c6` — an ancestor of `main`, so the branch has not advanced past its base — with 21 changed paths in the working tree (`git -C .claude/worktrees/agent-ad2893b1fba4d7f5b status --short | wc -l`). Nothing of it is landed, so do not read the current tree as evidence of what the numerical policies will be; check the branch before deriving a reduction order from them.

## Closes when

1. A single-workgroup reduction and a multi-pass reduction both exist as schedule alternatives beside the serial one, and the portfolio can retain all three for one program.
2. The empty domain has a stated identity element per reducer, and an extent-0 reduction produces it in every strategy — the reference case is emittable today (`emit-an-empty-domain-reduction-to-metal`, `done`), so this is testable rather than blocked.
3. The accumulation dtype is an explicit part of the strategy, not inherited silently from the element dtype, and a strategy that would accumulate at a narrower width than the contract allows is rejected with a typed reason.
4. Deterministic and relaxed orders are distinct alternatives whose legality is checked against the declared realization, and **reassociation and contributor permutation stay independent** — a tree reduction needs reassociation, while an atomic or nondeterministic-arrival combine also needs permutation, and a strategy must not be admitted by checking one and using both. Note the asymmetry this ticket has to resolve: `ReductionTopology::Serial` carries both `permits_reassociation` and `permits_permutation` (`crates/tiler-ir/src/schedule/model.rs:349-351`), but `NumericalRealization` carries `contraction` and `reassociation` and **no permutation permission** (`crates/tiler-ir/src/schedule/numerics.rs:110-120`), so there is currently no declared source for the topology's permutation flag to be derived from. Either the realization gains the permission or the flag's authority is stated explicitly; it must not be defaulted.
5. Synchronization and partial storage are explicit physical contracts of the multi-pass strategy: which pass writes what, where it lives, and what barrier or dispatch boundary makes it visible — never implied by the pass count.
6. Feasibility is separate from cost: a strategy the target cannot honour (threads per workgroup, local memory, barrier availability, a permission the contract withholds) is rejected with an explainable reason rather than costed to infinity, and numerical evidence for each admitted strategy is recorded at its own evidence class.
7. Selection may deliberately return multiple kernels for one reduction, the explain output says why the multi-kernel plan won, and `make full` passes.

## Graph maintenance

- **You will probably fire the enforcers trigger, on purpose.** A multi-pass reduction writes partials one pass consumes — a per-region boundary variation the bounded profile has never produced. When `the_bounded_profile_admits_no_undischarged_boundary` fails under your change, that is the designed signal (its message names the mismatch): append the mismatch to `implement-boundary-property-enforcers` as its first real case and tell the coordinator it is startable. Do NOT repair the test by widening the property sets.
- **The permutation-permission asymmetry (criterion 4) must be resolved, not defaulted.** If you give `NumericalRealization` a permutation permission, that widens a type the whole physical path carries — coordinate with whatever `rebase-and-land-the-stranded-numerical-policies-worktree` lands first, since its worktree already widens the dimension vocabulary 4→11.
- **Accumulation-dtype rejection** (criterion 3) is a new typed refusal: give it an explain record and update the census in the same commit.
