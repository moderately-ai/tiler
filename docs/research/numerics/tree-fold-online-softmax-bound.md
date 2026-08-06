---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.tree-fold-online-softmax-bound"
kind: "research"
title: "The tree-fold form of the online-softmax rescaling bound"
topics: ["numerics", "accuracy", "proof", "reductions", "scheduling"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["sound-proof", "bounded-measurement"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.optimizer"]
depends_on: ["tiler.research.numerics.certified-bounds-as-rewrite-permissions", "tiler.research.numerics.elementary-identity-rewrite-dimension", "tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound"
---

# The tree-fold form of the online-softmax rescaling bound

**Status:** derivation complete and measured. Nothing here is a contract change, no permission is admitted or proposed, and no accepted decision is reopened. This record extends [the certified-bounds record](certified-bounds-as-rewrite-permissions.md)'s Part 2 to the fold shape a parallel schedule actually selects; that record's derivations are cited evidence and are not edited here beyond one dated pointer at the open axis this record closes.

## Traceability

- **Current disposition:** pending. No ADR adopts this record and no contract sentence has moved for it.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md) would own any tolerance vocabulary and [the optimizer model](../../compiler/optimizer.md) where the admission rule sits, exactly as the certified-bounds record states; neither is edited here, and `contracts/numerics` was outside this ticket's scopes.
- **Extends:** [Certified rounding-error bounds as rewrite permissions](certified-bounds-as-rewrite-permissions.md), whose Part 2 derives the sequential case, states that it is the sequential case, and files the tree case as an open axis. Every classical result used below is the one that record reads in the vendored [Acta Numerica survey](sources/README.md#acta-numerica-fp-2023), cited from there rather than re-sourced.
- **Composes with:** [The elementary-identity rewrite dimension](elementary-identity-rewrite-dimension.md) and [ADR 0101](../../decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md), which name the second freedom the fold consumes; [Reduction semantics and legality](reduction-semantics-and-legality.md), whose allowed-trees table supplies the third; [ADR 0096](../../decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md) and [ADR 0100](../../decisions/0100-admit-the-multi-round-two-level-reduction-composition.md), whose composed round/subgroup/lane fold is the shape Part 4 instantiates against.
- **Evidence:** the executable witness at [`spikes/numerics/online_softmax_tree_bound/`](../../../spikes/numerics/online_softmax_tree_bound/README.md), which imports and cross-checks the retained sequential probe at [`spikes/numerics/online_softmax_bound/`](../../../spikes/numerics/online_softmax_bound/README.md).
- **Work record:** [`derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound`](../../../tickets/derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound.md).

## Outcome

**The bound is a function of the fold tree's *depth*, not of its contributor count, and the telescoping argument that made the sequential bound input-independent survives tree reassociation intact rather than in weakened form.** Three results carry that, and one of them refutes the conjecture the producing ticket wrote down.

**Inference — the price is `D`-parametric, where `D` is the rescale depth.** For a merge tree whose deepest root-to-leaf path crosses `D` rescaling merges, against the two-pass fold over the *same* tree,

```text
P(D, h2)  =  (1 + eps_exp)^D * (1 + gamma_{h2 + D}) / (1 + gamma_{h2})  -  1
          ~=  D * (u + eps_exp)                                to first order
```

where `h2` is the summation depth of that matched baseline. The sequential form is the instance `D = h2 = V - 1`, at which this reduces **exactly** — not approximately — to the certified-bounds record's `P(V)`. The probe checks that equality in exact rational arithmetic at five contributor counts and it is a check that has been watched failing.

**Inference — the telescoping survives, and it survives for a structural reason rather than a lucky one.** Along any root-to-leaf path of any merge tree, balanced or not, the subtree maxima are non-decreasing and the first of them already dominates the contributor, so the rescale-argument perturbations sum to `(m_V - x_j) * u` and nothing else. The logit spread `A` cancels between the two folds exactly as it does in the chain. **No dependence on `V`, on balance, or on the tree's shape enters the argument-perturbation term at all.**

**Inference — the ticket's conjecture that a tree is "more expensive in `exp` count per path" is wrong, and the correction is the whole quantitative result.** A merge node evaluates two exponentials, but only *one of them lies on any given contributor's path*; the other rescales the sibling. Per level, a merge and a sequential step are indistinguishable — one multiply, one add, one exponential, one argument perturbation — so the tree is cheaper in *both* terms and by the same factor. At `V = 512` the derived price falls from `6.09e-5` to between `3.58e-7` and `1.79e-6` depending on the shape, a tightening of **34x to 170x**; at `V = 8192` it is 130x to 1366x. A tree fold is not a cheaper `gamma` bought with a dearer elementary term. It is uniformly cheaper.

**Inference — the tree form consumes a third numerical dimension the sequential form does not, and that is a gate, not a cost.** A tree-shaped rescaling fold additionally consumes **reassociation**, because reaching a balanced grouping from the pinned strict left fold is a regrouping that neither distributivity nor the elementary identity performs. This matters in the admitting direction and in the refusing one: it is the reason the price must be quoted against a *shape-matched* baseline, and the reason a reader must not read the tree fold's smaller absolute bound as a free improvement.

## Part 1 — The fold, stated before it is analysed

**Fact — the merge operator, as the producing ticket states it and as a parallel realization spells it.**

```text
leaf(block)                   = (m_b, d_b)  where m_b = max over the block and
                                d_b = sum_{i in block} exp(x_i - m_b)
merge((m1,d1), (m2,d2))       = (M, d1 * exp(m1 - M) + d2 * exp(m2 - M)),  M = max(m1, m2)
```

A leaf is a **block-local two-pass**: its own maximum, then its own sum. That covers every case the shape space contains without a special rule — a lane's serial prefix, a staged tile, and a single contributor (a block of size one, whose `d` is `exp(0)`) are the same object at different block sizes. The whole fold is then a tree of merges over the leaves.

**Fact — over the reals the tree computes the two-pass normalizer, for any tree.** By induction on the tree, `d_nu = sum_{j in S_nu} exp(x_j - m_nu)` at every node `nu` with leaf set `S_nu`: the merge substitutes `d_1 exp(m_1 - M) + d_2 exp(m_2 - M)` and each `exp(x_j - m_i) exp(m_i - M) = exp(x_j - M)`. At the root `m_root = m_V` because `max` is associative, commutative, and exact, so `d_root = sum_j exp(x_j - m_V)`. This is the tree analogue of Milakov and Gimelshein's Theorem 1 and, like it, **it is a statement over the reals.** In binary32 it is false, which is what the rest of this record prices.

**Inference — the freedoms consumed, and the third one is new.** The certified-bounds record derives that the sequential form consumes distributivity (each rescale factor multiplies through a partial sum) and the exponential's functional equation, and `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` in `crates/tiler-ir/src/semantic/softmax.rs` states both. A **tree** form consumes those two and reassociation as well. The derivation is a chain and each link is separately checkable: expanding the tree fold over the reals yields the sum of the two-pass contributors *grouped by that tree* — distributivity and the elementary identity get exactly that far and no further, because distributing `r` over `a + b` preserves the association of the sum it distributes over, and the elementary identity changes how many evaluations there are rather than how a sum is parenthesized. Reaching the pinned contributor order from a tree grouping is then precisely the move [the reduction contract's allowed-trees table](reduction-semantics-and-legality.md) governs, and `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` pins the strict left fold, so the regroup consumes the separately resolved reassociation permission.

**The sequential form is the boundary case that confirms this rather than contradicting it.** Algorithm 3 is the *left-deep* merge tree, whose expansion is the left-deep grouping — the canonical one. It therefore needs no reassociation, which is why the registered fact naming exactly two freedoms is correct as written and this is an addition for the tree case rather than a correction of it.

The maximum pass contributes no gate: `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` records that the pinned extrema family is associative and commutative on every binary32 input, so the tree of maxima consumes no permission at all.

## Part 2 — The derivation

**Assumptions, each of which the admission rule must discharge rather than inherit.** Binary32, round-to-nearest-ties-to-even, unit roundoff `u = 2^-24`. No overflow. **No subnormal intermediate**, because (2.5a) does not hold there; the probe counts them and refuses rather than reporting a bound the model does not support. `max` is exact, as a selection among representable values. The `exp` routine has relative error at most `eps_exp`, taken from the target's declared realization. The side condition is `N * u < 1` where `N` is the rounding count derived below.

### Step 1 — what one contributor passes through

Fix contributor `j`, in block `b`, and let its root path cross the merge nodes `nu_1, ..., nu_D` with subtree maxima `M_1 <= M_2 <= ... <= M_D = m_V`. The maxima are non-decreasing because the subtrees are nested, and `x_j <= m_b <= M_1`.

**Its own block's term.** One subtraction `fl(x_j - m_b)`, one `exp`, and the block's summation, which places it at depth `h_intra(j)` in the block's own sum tree — `L - 1` for a serial prefix of `L`, `ceil(log2 L)` for a pairwise tree.

**Each merge above it.** At `nu_i` the state carrying `j` is rescaled by `exp(m_{nu_{i-1}} - M_i)`, which costs one subtraction, one `exp`, one multiply, and then one add. The sibling's rescale factor costs the machine two more operations and costs *contributor `j`* nothing, because it never touches `j`'s value. **This is the step the producing ticket's conjecture got wrong**: the per-merge operation count is two multiplies, two exponentials, and one add, and the per-*path* count is one of each.

So, taking maxima over contributors:

```text
E   =  1 + D                      elementary evaluations on the deepest path
N   =  h_intra + 2D               multiply/add roundings on it
h2  =  h_intra + D                the matched two-pass fold's adds on it
```

with `D` the rescale depth. On a ragged tree the three maxima may be attained at different contributors; taking each independently is still a valid upper bound, because every contributor's own factor is dominated term by term. The probe's `v512-ragged-caterpillar` shape exists to exercise that case rather than let it be assumed.

### Step 2 — the telescoping, which is the load-bearing step

Contributor `j` carries one argument perturbation per exponential on its path. The block term's is `|x_j - m_b| * u = (m_b - x_j) u`; the merge at `nu_i` contributes `(M_i - M_{i-1}) u` with `M_0 := m_b`. Because the maxima along the path are non-decreasing and `M_D = m_V`, they sum exactly:

```text
(m_b - x_j)  +  sum_{i=1..D} (M_i - M_{i-1})   =   (m_b - x_j) + (m_V - m_b)   =   m_V - x_j   <=   A
```

**The telescoping survives tree reassociation with no weakening and no residual dependence.** The sequential argument used a chain of consecutive running maxima; this one uses a chain of nested subtree maxima, and the only property either needs is monotonicity along the path. It holds for a balanced tree, a caterpillar, a ragged block partition, a multi-round loop-carried accumulator, and any composition of them, because *every* root path in *every* merge tree has non-decreasing maxima by nesting alone. Nothing in the argument mentions `V`, the tree's height, its balance, or its branching.

Since `exp(a(1+delta)) = exp(a) exp(a delta)`, the product of the path's perturbation factors is bounded by `exp(A*u)` — **the same factor the two-pass fold carries**, so it cancels from the price exactly as it does in the sequential case.

### Step 3 — the two bounds

Every contributor's exact term is positive, so `sum |t_j| = sum t_j` and the forward relative error is the largest per-term relative deviation. Collecting the three factors:

```text
B_tree  =  (1 + eps_exp)^E   * exp(A*u) * (1 + gamma_N)   -  1
B_2     =  (1 + eps_exp)     * exp(A*u) * (1 + gamma_{h2}) -  1
```

`B_2` is the two-pass fold summed over the *identical* tree: a global maximum, one `exp` per contributor, and the same grouping of additions. Using `gamma` at a tree height rather than at a contributor count is (4.8)'s own statement — "`h` is the height of the binary tree that underlies the chosen summation algorithm" — so the baseline needs no new argument.

**Where that statement actually comes from, recorded 2026-08-06 because this derivation is the one place in the tree that leans on the `h` form rather than on an endpoint.** The sentence is the Acta Numerica survey's, and the survey cites Higham (2002) §4.2 for it. The monograph was bought and read on that date, and **§4.2 does not contain it**: the words "tree" and "height" do not appear in its Chapter 4, which proves the two endpoints instead — `gamma_{n-1}` for its fully general summation algorithm under any parenthesizing, and `gamma_{log2 n}` for pairwise summation at `n = 2^r`. **The derivation above is unaffected and no step moves**, because §4.2 supplies the argument the `h` form follows from directly: its pairwise bound is derived by observing that each addend takes part in `log2 n` additions, and a leaf's root-path length in an arbitrary merge tree is that same count bounded by the tree's height. So the `h` form is a sound generalization of Higham's own mechanism, stated by four authors of the field in a peer-reviewed survey; what changes is only that a reader following the citation to "§4.2" will find the mechanism rather than the sentence. The verdicts are at [the certified-bounds record's](certified-bounds-as-rewrite-permissions.md) (4.8) bullet and the reading evidence at [the `higham-asna-2002` row](sources/README.md#higham-asna-2002).

### Step 4 — the price, and the baseline that may legitimately be used

```text
P  =  (1 + B_tree) / (1 + B_2)  -  1
   =  (1 + eps_exp)^(E-1) * (1 + gamma_N) / (1 + gamma_{h2})  -  1
   =  (1 + eps_exp)^D * (1 + gamma_{h2 + D}) / (1 + gamma_{h2})  -  1
   ~= D * (u + eps_exp)                                 to first order
```

`exp(A*u)` is absent, so the price is instantiable at compile time from the fold tree and the target profile with no knowledge of any input value — the property that made the sequential bound usable, preserved.

**Fact — the sequential form is the instance `D = h2 = V - 1`.** Then `E = V`, `N = 2(V-1)`, and `P` becomes `(1 + eps_exp)^(V-1) (1 + gamma_{2(V-1)}) / (1 + gamma_{V-1}) - 1`, character for character the certified-bounds record's formula. The probe asserts this as an exact rational equality against that record's own retained implementation, at `V` in `{2, 8, 64, 512, 8192}`, and the assertion has been watched failing under two separate perturbations.

**Inference — the baseline must be shape-matched, and quoting an unmatched one is unsound in the admitting direction.** Against a *sequential* two-pass baseline (`h2 = V - 1 = 511`) the tree online fold's price at `V = 512` is `-2.88e-5`: a credit, because `gamma_18` is far below `gamma_511`. **That number must never reach an admission rule.** The only contract under which the tree online fold is legal is one that permits reassociation, and under that same contract the tree-summed two-pass fold is legal too and is the alternative the rewrite actually displaces. A rule that priced the tree fold against a baseline the caller's contract also forbids would be crediting the rewrite for a permission the caller spent elsewhere. **The matched baseline is not a convention; it is what keeps the price a statement about the rewrite.**

### Step 5 — the unbalanced case

The derivation never assumed balance. `D`, `N`, and `h2` are maxima over root paths, and every step above is per-contributor. An unbalanced tree takes the depth of its deepest path, a ragged block partition takes the largest block's intra depth, and a tree mixing both takes each maximum independently. The `v512-ragged-caterpillar` shape — blocks of `(1, 1, 2, 4, 8, 16, 32, 64, 128, 256)` merged by a caterpillar — is measured and its bound is not violated.

Two side conditions move in the tree's favour and are worth stating because a reader would expect the opposite. `gamma_N` requires `N * u < 1`, which caps the sequential fold at `V <= 2^23` and is not remotely reachable by a tree (`N = 18` at `V = 512`). And the first-order price is now well below `2^-20` at every schedulable shape, so the second-order terms `gamma` carries — the reason the certified-bounds record kept `gamma` over the sharper `n*u` constants — matter even less than they did.

**One side condition moves against the tree, and it is the one to watch.** A tree over `V` contributors performs `2(V-1)` rescale evaluations and products where the sequential chain performs `V-1`. The *depth* obligation shrinks; the *number of sites* at which "no subnormal intermediate" and "no overflow" must hold roughly doubles. The bound does not see this — it is a per-path quantity — but the admission rule's first obligation does, and a shape that halves the price while doubling the subnormal exposure has not made the rewrite unconditionally safer.

## Part 3 — Instantiation at the shapes attention uses

**Measurement** — every row below is computed in exact rational arithmetic by [the probe](../../../spikes/numerics/online_softmax_tree_bound/README.md) at `eps_exp = u = 2^-24`, and is reproduced in its `results.json`. `intra/merge` names the grouping inside a block and across blocks.

| Shape | `V` | blocks | intra/merge | `D` | `h2` | `E` | `N` | derived price | vs. sequential |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `v512-alg3-serial` | 512 | 512 | balanced/serial | 511 | 511 | 512 | 1022 | `6.091919E-5` | 1.0x |
| `v512-binary-tree` | 512 | 512 | balanced/balanced | 9 | 9 | 10 | 18 | `1.072885E-6` | 56.8x |
| `v512-block16-tree` | 512 | 32 | balanced/balanced | 5 | 9 | 6 | 14 | `5.960468E-7` | 102.2x |
| `v512-block32-tree` | 512 | 16 | balanced/balanced | 4 | 9 | 5 | 13 | `4.768374E-7` | 127.8x |
| `v512-block64-tree` | 512 | 8 | balanced/balanced | 3 | 9 | 4 | 12 | `3.576280E-7` | 170.3x |
| `v512-block32-serial-outer` | 512 | 16 | balanced/serial | 15 | 20 | 16 | 35 | `1.788142E-6` | 34.1x |
| `v512-block64-serial-intra` | 512 | 8 | serial/balanced | 3 | 66 | 4 | 69 | `3.576286E-7` | 170.3x |
| `v512-one-block` | 512 | 1 | balanced/balanced | 0 | 9 | 1 | 9 | `0` | control |
| `v512-ragged-caterpillar` | 512 | 10 | balanced/serial | 9 | 9 | 10 | 18 | `1.072885E-6` | 56.8x |
| `v64-alg3-serial` | 64 | 64 | balanced/serial | 63 | 63 | 64 | 126 | `7.510234E-6` | 1.0x |
| `v64-binary-tree` | 64 | 64 | balanced/balanced | 6 | 6 | 7 | 12 | `7.152562E-7` | 10.5x |
| `v64-block8-tree` | 64 | 8 | balanced/balanced | 3 | 6 | 4 | 9 | `3.576280E-7` | 21.0x |
| `v64-one-block` | 64 | 1 | balanced/balanced | 0 | 6 | 1 | 6 | `0` | control |

**Four readings, in descending order of consequence.**

**The block size `B` does not appear in the price; the block *count* does.** At `V = 512` the three block sizes the ticket names give `D = 5, 4, 3` for `B = 16, 32, 64` — and the driver is `log2(V/B)`, the number of merge levels, not `B`. The ambiguity in whether "B" means block size or block count dissolves rather than needing a ruling: `16 x 32` and `32 x 16` both appear in the table and the formula reads the tree either way. **The intra-block fold is common to both folds and cancels entirely**, which `v512-block64-serial-intra` demonstrates at the extreme: a 63-deep serial prefix inside each block raises `h2` from 9 to 66 and leaves the price at `D = 3` unchanged to six digits.

**`D * (u + eps_exp)` is not merely a first-order approximation at these shapes; it is the answer.** With `eps_exp = u`, `2Du` reproduces every price at `D <= 15` to six significant figures and the `v64-alg3-serial` row at `D = 63` to five; only at `D = 511` does the exact form separate in the fifth digit (`6.091595E-5` against `6.091919E-5`). The second-order structure `gamma` carries is what a caller pays for at a sequential chain and stops paying for at a tree.

**The flash-realistic shape is the expensive one, and it is still 34x cheaper than the chain.** `v512-block32-serial-outer` — blocks reduced in parallel, then merged by a sequential outer loop, which is what a loop-carried streaming schedule does — has `D = 15` rather than `4`, because a sequential outer loop is a caterpillar and its depth is the block count minus one. **The rescale depth is a property of the schedule's *merge topology*, and a schedule that parallelizes the inner fold while serializing the outer one keeps the larger of the two depths.** At `V = 8192` with 128-wide blocks the same contrast is `D = 63` (price `7.510236E-6`, 130x tighter than the chain's `9.772783E-4`) against `D = 6` for a fully parallel merge (price `7.152563E-7`, 1366x).

**The control earns its place.** `v512-one-block` and `v64-one-block` have `D = 0`, a derived price of exactly `0`, and an observed divergence of exactly `0` on all fourteen of their rows — because with one block the online fold *is* the two-pass fold. A defect that produced any divergence there would be caught against zero rather than against a bound with headroom.

## Part 4 — The shape parameter a schedule must hand the admission rule

**Inference — it is the pair `(D, h2)`, and both are combinatorial properties of the scheduled fold tree.** Neither depends on an input value, so both are computable at compile time from the schedule and the target profile alone, which is the property [ADR 0095's second reopening condition](../../decisions/0095-decline-a-distributivity-permission.md) requires of "a bound derived at the fold shape a parallel schedule would select". `V` alone is not sufficient and is not even necessary: two schedules over the same 512 contributors differ in price by 170x, and `v512-binary-tree` and `v512-ragged-caterpillar` have the same price over different contributor partitions.

**How the pair reads off the accepted composition vocabulary.** [ADR 0100](../../decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) composes a fold over the lexicographic `(round, subgroup, lane)` block index, with [ADR 0096](../../decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md)'s two levels inside each round. Depths compose additively along a path, so for a composition with `W` lanes per subgroup, `G` subgroups, `R` rounds, and a per-lane prefix of `k` contributors:

```text
h2  =  depth(per-lane prefix)  +  log2 W  +  log2 G  +  (R - 1)
D   =  rescale levels among exactly those same levels
```

and `D` differs from `h2` in precisely one way: **a level contributes to `D` only if the partial states entering it carry different maxima.** A lane's serial prefix contributes `k-1` to `D` if the lane runs the online recurrence element by element, and `0` if it computes a lane-local maximum and then a lane-local sum. That is a schedulable choice with a real numerical consequence and no other visible difference, which is exactly the kind of choice a bound has to be able to see.

**What this means for the certified-bounds record's third admission obligation.** That obligation already requires the evidence identity to bind "fold height" among the complete scheduled candidate. This derivation widens it in one concrete way rather than replacing it: **the identity must bind the shapes of *both* folds** — the rewrite's merge tree and the baseline it is priced against — because the price is a two-shape quantity and a bound derived against one baseline does not admit the same candidate against another. The unmatched-baseline credit in Part 2 Step 4 is what that widening prevents.

**And one obligation that does not move.** Discharging "no subnormal intermediate" is now a check over roughly twice as many sites, as Part 2 Step 5 notes, but it is the same check with the same refusal. Nothing about the tree shape makes an undischarged side condition safer.

## Part 5 — A distinction the sequential record's probe conflates, reported rather than smoothed

**Inference.** `P` is the ratio of the two folds' *bounds*, so what it bounds is the additional relative budget the rewrite consumes against the shared real reference: `1 + B_tree = (1 + B_2)(1 + P)` holds by construction. It is **not** an upper bound on the realized divergence `|online - two_pass| / R`, because the two folds' errors are independent perturbations that may land at opposite ends of their brackets; the rigorous bound on that quantity is `B_tree + B_2`, which is about twice `P` at every shape in the table.

The retained sequential probe checks the observed divergence against `P` and passes on all 22 of its cases, and this probe reproduces that behaviour on all 91 of its rows. **The check is a good detector and a bad theorem.** It fired on 55 of 91 rows under a sign-flipped rescale factor and on 20 under a hundred-fold understatement of the price, so it earns its place; but a corpus that ever violated it would refute nothing, and the certified-bounds record's Part 2 phrasing — "its price over the two-pass fold it replaces is at most ..." — reads as the theorem rather than as the budget statement it actually is.

**Nothing in Part 3 of that record depends on the distinction, and saying so is part of reporting it.** The admission rule compares a rewrite's *bound* against a caller's tolerance, and `B_tree` is that bound; `P` is presentational, and is sound in its intended reading. What needs a sentence somewhere durable is which reading is meant. This probe therefore carries both checks, labelled, and [`separate-the-rescaling-price-from-the-observed-fold-divergence`](../../../tickets/separate-the-rescaling-price-from-the-observed-fold-divergence.md) owns the wording, because `contracts/numerics` and that record's derivations were outside this ticket's scopes.

## Part 6 — The measurement, and what it cannot do

**Measurement.** 91 rows — 13 declared fold shapes crossed with the 14 logit sets of matching contributor count from the retained adversarial corpus — evaluated in exact binary32 semantics against a 120-digit reference on CPython 3.11.13 / arm64 / Darwin, 2026-08-06. No derived bound is violated. No subnormal intermediate is reached. The general price equals the retained probe's sequential price exactly at all five specialization counts. The left-deep merge tree reproduces the retained probe's online and two-pass folds **bit for bit** on all 14 of its rows. Bound-over-observed ratios run from 9.5 at `v64-one-block` to `2.7e19` where every term but one underflows the sum.

**Six perturbations were watched failing**, each in a scratch copy, with the unperturbed copy returning exit 0 before and after: a flipped rescale sign, a hundred-fold understated price, a deleted shape, an off-by-one price exponent that only the specialization cross-check can catch, a spread that drives `exp` into the subnormal band, and — the important one — substituting `ceil(log2 blocks)` for the merge tree's actual rescale depth.

**That last perturbation failed on 1 row out of 91, and it is the honest limitation of this evidence.** Understating `D` by 56x at the sequential shape leaves 90 rows passing, because the bound is loose by one to nineteen orders of magnitude over this corpus and absorbs the error whole. **This population refutes structural errors in the fold; it does not pin the shape parameter.** The counting argument in Part 2 is carried by the derivation and by the exact specialization to the already-published sequential case, and no reading of `results.json` should suggest the numbers confirm it. The looseness itself is the same phenomenon the certified-bounds record measured and for the same reason: `gamma` is attained only when every rounding goes the same way, and a sum of positive terms of comparable magnitude does not approach it.

**And the evidence classes stay apart.** The derivation is `sound-proof` in [the metadata contract's](../../document-metadata.md#kinds-and-status-facets) sense — it follows within the documented model and its stated assumptions — and it is a pencil derivation no machine has checked, resting on classical results read in a peer-reviewed restatement and, since 2026-08-06, checked against the monograph that proves them — a check that moved no step here and produced the attribution note at Step 3. The probe is `bounded-measurement`: a Python simulation of binary32 with a correctly rounded `exp` that no real target provides, on one host, over a finite declared population.

## Open axes, each with a filed destination

- **The price is stated as a bound on the extra budget and checked as though it bounded the divergence.** → [`separate-the-rescaling-price-from-the-observed-fold-divergence`](../../../tickets/separate-the-rescaling-price-from-the-observed-fold-divergence.md).
- **The rescale depth is a schedule property no schedule type carries.** A fold's merge topology decides `D`, and neither the cooperative topology nor the round composition states it; a bound instantiated from a shape the schedule does not declare would be instantiated from an assumption. → [`carry-the-rescale-depth-a-parametric-bound-instantiates-from`](../../../tickets/carry-the-rescale-depth-a-parametric-bound-instantiates-from.md), filed `deferred` behind the permission gates below, because a shape parameter for a rewrite no contract can perform is a field nothing reads.
- **Whether a target's `exp` returns exactly `1` at a zero argument** decides whether the winning side of every merge is exact. This derivation charges `eps_exp` and a multiply on both sides and is therefore conservative; the sharpening would remove roughly half of `E` and is worth exactly one measurement. Metal's Table 8.1 states `exp <= 4 ulp`, which does **not** imply exactness at zero, so the conservative reading is currently required rather than merely chosen. → [`measure-whether-a-targets-exponential-is-exact-at-zero`](../../../tickets/measure-whether-a-targets-exponential-is-exact-at-zero.md), filed `deferred` with its trigger.
- **The numeric `eps_exp` was not retrievable from the authority that owns it**, which this record inherited unchanged from the certified-bounds record. → **Closed 2026-08-06** by [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](../../../tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md): `elementary_relative_accuracy` answers with the requirement-side number and the region it holds over; the certified-bounds record's open axis carries the numbers and [the rule-object record's](online-softmax-rule-object.md) Part 4 obligation 2 the full reading.
- **The sharpened summation constants would tighten this bound exactly as they would the sequential one**, and the tree case does not change that trade. → [`tighten-the-rescaling-bound-with-the-sharpened-summation-constants`](../../../tickets/tighten-the-rescaling-bound-with-the-sharpened-summation-constants.md), already `deferred` with its trigger.
- **Higham's monograph was unread, and this record inherited the dependency at proof level rather than at statement level.** → **Closed 2026-08-06**: bought, read at §3.4 and §4.2, and recorded `metadata-only` with a digest at the `higham-asna-2002` row in [the source record](sources/README.md#higham-asna-2002). One finding reaches this record, and it is the `h`-form attribution noted at Step 3 above rather than anything in a bound: no step moves, and the classical results this derivation uses are the ones §4.2 proves.

## What this record establishes, parks, leaves to experiment, and reconsiders

**Closes.** The tree-fold bound is derived, its telescoping argument is re-established rather than assumed, the unbalanced case is covered by construction rather than excluded, the comparison against the sequential form is quantified at the shapes attention uses, and the shape parameter an admission rule needs is named and shown to be compile-time computable. The certified-bounds record's "the tree-fold form of the bound is underived" axis is answered.

**Parks.** The price-versus-divergence wording, the schedule field that would carry `D`, and the `eps_exp` retrieval gap — each filed above, none absorbed silently. No contract sentence is moved and no ADR is edited by this record.

**Experiment remaining.** One measurement, bounded and cheap: whether a target's exponential is exact at a zero argument. It changes the constant and not the form, and its trigger is the first target profile that declares an elementary accuracy a bound can instantiate from.

**Reconsideration trigger.** [ADR 0095's second reopening condition](../../decisions/0095-decline-a-distributivity-permission.md), added by the 2026-08-06 reaffirmation, is a conjunction of three clauses: a rule in the certified-bounds admission shape, a retrievable `eps_exp`, and **the bound derived at the fold shape a parallel schedule would select**. **This record satisfies the third clause and no other.** The condition has therefore *not* fired: no rule exists in that shape, and `eps_exp` is still not retrievable from the target authority. What has changed is that the third clause is no longer the blocking one, and that a joint admission — this dimension together with [ADR 0101's](../../decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) reserved elementary-identity permission — would now be priced at a schedulable shape rather than at a shape no kernel folds. Whether that reopens either decision is Tom's, on evidence that now includes this record; nothing here reopens or relitigates a reaffirmed decline.

**One thing the reopening analysis must carry that it did not before.** The parallel form consumes **three** dimensions, not two. Reassociation is already grantable — a permit-reassociation contract is registered — so this adds a conjunct to check rather than a third blocked door, but a joint decision that priced only distributivity and the elementary identity would be pricing the sequential fold while describing the parallel one.

## What this record does not establish

- **No contract changed and no permission moved.** No ADR is accepted, edited, or reopened; no crate changed; `implementation_status` is `not-started` and that is the honest value.
- **Nothing here establishes that a tree fold is faster, or that the online rewrite is wanted.** It establishes what a tree-shaped rescaling fold costs numerically relative to the two-pass fold at the same shape. Whether any of these shapes is profitable on any target is a cost-model and scheduling question no profile in this repository can answer.
- **A smaller bound is not a smaller error.** Every comparison in Part 3 is between two derived worst cases; the measured errors are one to nineteen orders of magnitude below both, and the ordering of bounds does not order the realized errors.
- **The measurement is a bounded observation of 91 rows in a Python simulation of binary32**, on one host, with a correctly rounded `exp` no real target provides. It refutes; it does not confirm. It explicitly does not pin the shape parameter, which the fourth watched perturbation demonstrates rather than leaves as a caveat.
