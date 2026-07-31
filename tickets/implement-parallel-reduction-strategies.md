---
id: implement-parallel-reduction-strategies
title: Implement parallel reduction strategies
status: todo
priority: p1
dependencies: [implement-first-profile-numerical-policies, implement-analytical-component-cost-model, calibrate-and-activate-parallel-reduction-selection]
related: [admit-the-rms-normalization-family, admit-the-softmax-family, scope-transformer-nonlinear-normalization-and-reductions]
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reduction, scheduling, numerics]
---
## User-visible outcome

A reduction can be scheduled as a single-workgroup or multi-pass strategy—not only the serial order—with the numerical legality of each order checked against the declared realization. Larger reductions stop being serialized by default only after measured calibration demonstrates a faster valid strategy on the qualified target.

Add single-workgroup and multi-pass reductions beyond the serial schedule. Define empty identities, accumulation dtype, deterministic/relaxed orders, synchronization, partial storage, feasibility and numerical evidence; selection may deliberately choose multiple kernels.

## Implementation keys

Treat this ticket as the rollup over the split dependency graph: target-neutral multi-pass, cooperative workgroup dataflow, typed synchronization authority, synchronized single-workgroup scheduling, Metal realization, and measured selection calibration. Preserve serial throughout, keep numerical permissions independent, and close only when the separately verified outcomes compose on the merged tree.

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

## Workload evidence from L3′ (2026-07-31)

The [transformer non-linear, normalization, and reduction derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) is where the accumulation-dtype and order findings for the selected language-model workload live. They are cross-linked rather than restated, because the contract belongs here and the workload evidence belongs there.

- **Criterion 3's accumulation dtype now has a named workload case.** The derivation records it as its decision **D-5** and assigns it to this ticket. One forward pass of the pinned workload contains 169 reductions the serial sum does not cover: 28 softmax occurrences covering 448·`T` rows and carrying two reductions each, plus 113 RMS normalization occurrences covering 729·`T` rows and carrying one each, over 144,384·`T` squared contributors in total. Three observations bear on the width and none decides it — the reference accumulates in F32, so F32 is what reproduces it; the L1 profile's measured sensitivity envelope attributes the dominant divergence on the C1 row to *contraction* order rather than to these reductions; and the 1024-contributor sum of magnitude-squared terms is the longest accumulation in the set and the one where a widening argument would be strongest at a longer context.
- **Criterion 4's independence has a concrete asymmetric case.** Softmax embeds two reductions with different order sensitivity in one operation: the row maximum is associative and commutative wherever its extrema family is total, so any tree over the same contributors gives identical bits, while the sum of exponentials is neither. A strategy selected for the pair must check the two passes separately — checking the maximum's freedom and applying it to the sum is precisely the "admitted by checking one and using both" failure this ticket already forbids, arriving through a single operation rather than through two.
- **The RMS normalization sum is a fused prologue, not a new reducer.** `mean(x²)` is `Sum` over an elementwise squaring, which is the shape `OrderedReduction` was defined for and which the correctness contract's reduction matrix already lists under "fused prologue and epilogue expressions". It needs a role, not a family.
- **An online single-pass softmax needs reassociation, and that is a legality fact rather than a cost one.** Rescaling a running sum when the running maximum changes regroups the contributor sequence, so the fused form most worth wanting is the one this ticket's permission checks have to gate.

## Workload evidence from L4 (2026-07-31)

The [attention program design](../docs/research/program-planning/first-attention-program-vertical.md) adds one correction and one asymmetry to criterion 3's evidence above; both belong here because this ticket owns decisions D-5 and D-6.

- **The longest accumulation in the workload is not the one the earlier evidence names.** The bullet above records the 1,024-contributor sum of squares as the longest in the non-linear set, and [the L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md)'s decision D-6 records the contraction profile's 1,024-to-3,072 contributor counts as "the longest accumulations in the workload". Both statements are true of the sets their authors measured. The attention block's value contraction, index structure `grts,gsd->grtd`, folds over `S` — 10 at the C1 prefill row, up to 18 across C1's decode, and up to 8,320 at the end of B1-d. So the workload's longest accumulation is a contraction over a symbolic, growing extent, and a widening argument made from the static extents alone is made from the wrong worst case.
- **Its conditioning differs in kind, which the width choice has to account for separately.** The contributors are probabilities in `[0, 1]` multiplied by values, and the probabilities sum to approximately one — a very different accumulation from a weight-activation dot product whose terms are of comparable magnitude and mixed sign. **Measurement — 111 of the 160 softmax rows at the C1 prefill shape sum to exactly `0x3f800000` and 49 do not**, so "approximately" is exact rather than rhetorical, and neither outcome is a property a check may assert.
- **The causal mask contributes exact zeros to that fold and their sign follows the value operand.** At the C1 shape a masked position contributes `+0.0 × v`, which is `-0.0` where `v` is negative, and a fold seeded at the first product can have its result's sign rewritten by the masked tail — **measured**: seed `0x80000000`, completed fold `0x00000000`, retained by [the attention-block probe](../spikes/program-planning/attention-block-reference/README.md). That interacts with criterion 5's partial storage and criterion 4's permutation independence: a strategy that partitions the contributor sequence differently across masked and attended regions is not merely reassociating. [`scope-causal-structure-aware-attention-schedules`](scope-causal-structure-aware-attention-schedules.md) owns the question; this ticket only needs to avoid admitting such a partition by accident.

## Graph maintenance

- **You will probably fire the enforcers trigger, on purpose.** A multi-pass reduction writes partials one pass consumes — a per-region boundary variation the bounded profile has never produced. When `the_bounded_profile_admits_no_undischarged_boundary` fails under your change, that is the designed signal (its message names the mismatch): append the mismatch to `implement-boundary-property-enforcers` as its first real case and tell the coordinator it is startable. Do NOT repair the test by widening the property sets.
- **Permutation and reassociation remain separate checks.** Perturb each permission independently and observe the corresponding strategy reject before restoring it.
- **Accumulation-dtype rejection** (criterion 3) is a new typed refusal: give it an explain record and update the census in the same commit.
