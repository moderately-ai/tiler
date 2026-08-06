---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.online-softmax-tree-bound"
kind: "experiment"
title: "Tree-fold online-softmax bound probe"
topics: ["numerics", "accuracy", "reductions", "scheduling", "measurement"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.tree-fold-online-softmax-bound"]
entrypoints: ["spikes/numerics/online_softmax_tree_bound_probe.py"]
last_verified: "2026-08-06"
ticket: "derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound"
---

# Tree-fold online-softmax bound probe

This probe checks the hand-derived worst-case bound on the online-softmax rescaling fold **when the `(m, d)` pairs are merged in a tree** rather than a chain, over a declared finite population of thirteen fold shapes crossed with the retained adversarial logit corpus. The derivation is in [the tree-fold record](../../../docs/research/numerics/tree-fold-online-softmax-bound.md); this directory holds only what was measured.

**What it establishes.** That the derived bounds are not violated on 91 (logit set, fold shape) rows; that the general price formula specializes *exactly* to the sequential price the retained probe already computes, at five contributor counts; that the degenerate left-deep merge tree over single-contributor leaves reproduces the retained probe's own online and two-pass folds **bit for bit**; and how loose the tree bounds are, which is the quantity that decides whether a caller's tolerance can be met at a schedulable shape.

**What it does not establish.** That the bounds hold universally. A finite population refutes a bound by exhibiting a counterexample and never proves one, which is the `Empirical` boundary [the region-accuracy contract](../../../docs/research/numerics/region-accuracy-contract.md) defines. The record labels the derivation `sound-proof` on its algebra and this probe `bounded-measurement` on its numbers, and the two claims are deliberately not merged.

## It imports the retained probe rather than copying it

[`online_softmax_bound_probe.py`](../online_softmax_bound_probe.py) beside it owns the binary32 simulation, the exact-rational `gamma`, the adversarial logit corpus, and the sequential price. This probe imports that module, so the two cannot drift apart in their arithmetic and two cross-checks become possible that a copy could not support: the general price instantiated at Algorithm 3's own counts must *equal* `rewrite_price(V)` exactly, and the left-deep merge tree must reproduce `online_normalizer` and `two_pass_normalizer` at the bit. Both are checked on every run and both are proved able to fail below. The retained probe is not edited and its own `results.json` provenance is untouched, which is why this is a second directory rather than a rewrite of the first.

Every operation is still computed exactly in `Decimal` at 120 digits and rounded once to binary32, so no host floating-point rounding participates; `exp` is correctly rounded to binary32, the strongest admissible implementation, so any observed looseness belongs to the fold rather than to the elementary function. Bounds are computed in exact rational arithmetic through `Fraction`, never through a host float.

## What a "shape" is here

A shape is an **explicit tree, not a formula.** The contributors are partitioned into blocks; each block forms one leaf state by a *block-local two-pass* — its own maximum, then its own sum over a declared grouping — and the leaf states are merged by the rescaling operator `(m1,d1) + (m2,d2) = (max, d1*exp(m1-max) + d2*exp(m2-max))` over a second declared grouping. A block of size one is the single-contributor leaf, so no shape needs a special case. Evaluation and depth counting walk the *same* structure, which is why a shape cannot be the tree it claims not to be; the independence that matters is between the combinatorial bound and the exactly evaluated arithmetic, and that is preserved.

The four counted shape parameters are maxima over contributors of the root-path counts: `exp_calls` (`E`), `roundings` (`N`), `baseline_adds` (`h2`, the matched two-pass fold over the identical tree), and `rescale_depth` (`D`). Each maximum is taken independently, which stays a valid upper bound on a ragged tree where two of them are attained at different contributors — `v512-ragged-caterpillar` is in the population to exercise exactly that.

## Reproduce

From this directory's parent, or from anywhere — the probe resolves its own path:

```sh
python3 spikes/numerics/online_softmax_tree_bound_probe.py
python3 -O spikes/numerics/online_softmax_tree_bound_probe.py
```

Standard library only — no dependency to pin and none claimed. Verdicts are explicit checks rather than `assert`, so optimized Python cannot discard them. Either command exits nonzero instead of publishing JSON when a derived bound is violated, when a subnormal intermediate is reached, when a cross-check against the retained probe moves, or when the evaluated population does not match the declared one.

[`results.json`](results.json) is the byte-for-byte output retained from both modes on the recorded host, verified identical between them with `diff`. It binds both probes' SHA-256, the oracle library and precision, the format with its unit roundoff and smallest normal, the `exp` implementation and its `eps_exp`, the declared and evaluated shape, logit-set and row counts, the five specialization counts, the per-shape parameter table, and the recorded Python implementation, version, machine, and system.

**Measurement boundary.** CPython 3.11.13 / arm64 / Darwin, 2026-08-06, 91 rows, 0 failures, both modes. This is a simulation of binary32 in Python with a correctly rounded `exp` that no real target provides; nothing here was executed on a GPU, and no row is a claim about any device.

## Watched failing — 2026-08-06

Six perturbations were applied in a scratch copy of both probe files rather than in place. The unperturbed copy returned exit 0 before and after each. Every check the probe performs is covered by at least one of them.

| Perturbation | Result |
| --- | --- |
| The rescale factor's sign is flipped in `merge`, `exp(max - m)` for `exp(m - max)` — a real defect in the rewrite | exit 1, `175 check(s) failed over 91 rows`: 55 `online tree error exceeds its derived bound`, 55 `divergence exceeds the sum of the two derived bounds`, 55 `divergence exceeds the derived price`, and 10 bit-exactness failures against the retained probe |
| The derived price is divided by 100 | exit 1, `25 check(s) failed over 91 rows`: 20 divergence failures and all 5 specialization mismatches |
| A declared shape is removed from `shapes()` | exit 1, `population mismatch: evaluated 84 rows, expected 91` and `shape population mismatch: declared 12 shapes, expected 13` |
| `ceil(log2 blocks)` is substituted for the merge tree's actual rescale depth — **the exact mistake the derivation exists to refute** | exit 1, but only `1 check(s) failed over 91 rows`, on `increasing-v512-span1/v512-alg3-serial`. See the limitation below; this is the weakest of the six and it is reported rather than presented as a success |
| The price's `(E-1)` exponent becomes `E`, which *loosens* the price so no row check can fire | exit 1, `5 check(s) failed`, all specialization mismatches — the cross-check against the retained probe is independently load-bearing |
| A logit set of spread 95 is added, driving `exp` into the binary32 subnormal band | exit 1, `6 check(s) failed over 95 rows`: 4 rows report 9 to 18 `subnormal intermediate(s) reached, where the relative-error model does not hold`, plus both population literals |

**What this population cannot detect, so the evidence is not overread.** The fourth perturbation is the finding, not the footnote. Substituting a balanced-tree height for a serial merge chain's real rescale depth understates `D` by 56x at `v512-alg3-serial` and by 3.75x at `v512-block32-serial-outer`, and **90 of 91 rows still pass**, because the derived bound is loose by one to nineteen orders of magnitude over this corpus and absorbs the error. This probe therefore refutes structural errors in the fold and gross understatements of the price; **it does not pin the shape parameter**, and no reading of `results.json` should suggest that agreement here confirms the counting argument. The counting argument is carried by the derivation and by the exact specialization to the sequential case, not by these numbers.

The retained probe's own two undetectable perturbations are inherited unchanged: removing the elementary-function factor, or the summation term, from a bound still passes over this corpus.

## The population, and why each member is in it

**Fourteen logit sets**, reused from the retained probe's corpus rather than restated — the seven cases at `V = 64` and the seven at `V = 512`. They were chosen because they exercise a moving, a frozen, an alternating, and a dominated running maximum, and a tree fold has no reason to want different logits, only different groupings of the same ones.

**Thirteen fold shapes**, crossed with the logit sets of matching contributor count, giving `7*9 + 7*4 = 91` rows.

- **`v512-alg3-serial`, `v64-alg3-serial`** — the left-deep merge tree over single-contributor leaves. This *is* Milakov and Gimelshein's Algorithm 3, and it is in the population to be compared bit for bit against the retained probe's sequential fold. Its counts are `E = V`, `N = 2(V-1)`, `h2 = V-1`, and its derived price is `6.091919E-5` at `V = 512`, which is the number the certified-bounds record already publishes.
- **`v512-binary-tree`, `v64-binary-tree`** — the pure pairwise merge tree, the smallest rescale depth reachable at all.
- **`v512-block16-tree`, `v512-block32-tree`, `v512-block64-tree`, `v64-block8-tree`** — block-local two-pass then a pairwise merge across blocks, at the block sizes a flash-class kernel picks. Both readings of "B" over `V = 512` are covered, because `16x32` and `32x16` both appear.
- **`v512-block32-serial-outer`** — the flash-realistic shape: blocks reduced in parallel, then merged by a *sequential* outer loop, which is what a loop-carried streaming schedule does and where the rescale depth stops being logarithmic.
- **`v512-block64-serial-intra`** — the opposite mix, a long serial prefix inside each block and a tree across them. `D = 3` with `h2 = 66`, which separates the two shape parameters from each other and shows the price tracking `D` while the absolute bound tracks `h2`.
- **`v512-one-block`, `v64-one-block`** — the control. One block means no merge occurs, so the online fold *is* the two-pass fold; the derived price is exactly `0` and the observed divergence is exactly `0` on all fourteen of these rows. A fold defect that produced any divergence here would be caught by a comparison against zero.
- **`v512-ragged-caterpillar`** — blocks of `(1, 1, 2, 4, 8, 16, 32, 64, 128, 256)` merged by a caterpillar, deliberately unbalanced in both dimensions, so the counted maxima are attained at different contributors and the independent-maxima step in the derivation is exercised rather than assumed.

## Traceability

- **Supported claim:** [The tree-fold form of the online-softmax rescaling bound](../../../docs/research/numerics/tree-fold-online-softmax-bound.md).
- **Generalizes:** [Certified rounding-error bounds as rewrite permissions](../../../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 2, whose sequential derivation this probe's `alg3-serial` shapes reproduce exactly, and whose [probe](../online_softmax_bound/README.md) this one imports.
- **Derivation sources:** the classical results are read in `acta-numerica-fp-2023` and the algorithm in `online-softmax-arxiv-1805.02867v2`, both recorded in [the numerics source manifest](../../../docs/research/numerics/sources/README.md).
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [`derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound`](../../../tickets/derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound.md).
