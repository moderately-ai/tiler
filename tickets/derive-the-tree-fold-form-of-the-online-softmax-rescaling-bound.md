---
id: derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound
title: Derive the tree-fold form of the online-softmax rescaling bound
status: done
priority: p3
dependencies: []
related: [connect-certified-rounding-error-bounds-to-rewrite-permissions, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, reductions]
---
## User-visible outcome

The online-softmax rescaling bound covers the fold shape a real flash-class kernel uses — a tree of `(m, d)` pair merges across lanes and workgroups — rather than only the sequential recurrence the published algorithm states.

## Why this exists

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) derives the bound for Algorithm 3 exactly as Milakov and Gimelshein state it: a sequential scan, giving `2(V-1)` roundings and `V` calls to `exp` along the worst path. It says explicitly that the derivation is the sequential case.

**Fact.** A parallel realization does not have that shape. It merges partial states pairwise: `(m, d) merge (m', d') = (max(m, m'), d*exp(m - max) + d'*exp(m' - max))`, which is what the two-level subgroup and workgroup reduction work would compose.

**Inference.** The bound does not transfer by substituting `ceil(log2 V)` for the height. A merge applies **two** multiplies, **two** exponentials, and one add, where a sequential step applies one multiply, two exponentials, and one add — so the per-level operation count differs. More importantly the telescoping argument that made the logit spread cancel must be redone, because along a tree path the rescale arguments are differences between *partial* maxima rather than a chain of consecutive ones. The telescoping is likely to survive, since the partial maxima along any root path are still non-decreasing, but **that is a conjecture in this ticket and not a result** — it is exactly the step a careless derivation would assume, and assuming it is how a bound becomes wrong rather than loose.

## What this ticket must produce

- The bound for a balanced binary merge tree of height `h`, with the per-merge operation count derived rather than assumed, and the telescoping argument either re-established or replaced.
- The comparison against the sequential form. A tree fold should be *cheaper* in `gamma` (height `log2 V` rather than `V-1`) and *more expensive* in `exp` count per path, and which term dominates is the answer a scheduler needs.
- The unbalanced case, or an explicit statement that the bound is stated for balanced trees and that an unbalanced one takes the height of its deepest path.
- An extension of [the existing probe](../spikes/numerics/online_softmax_bound_probe.py) to the tree fold, with its corpus counted against a declared literal and at least one watched failure, matching the discipline the sequential probe already carries.

## Non-goals

Selecting a schedule; implementing a kernel; deriving bounds for fold shapes no capability set names.

## Closes when

The tree-fold bound is derived and checked, the record's sequential-only caveat is replaced by a statement covering both, and any divergence between the two forms is stated with the shape that decides which is preferable.

## Outcome — 2026-08-06

**Why this landed as it did.** This ticket's own conjecture was that the telescoping "is likely to survive" and that a tree would be cheaper in `gamma` and dearer in `exp` count per path. The first is now derived rather than conjectured; **the second is refuted**, and refuting it is what makes the result usable. A merge node evaluates two exponentials, but only one lies on any given contributor's root path — the other rescales the sibling — so per level a merge and a sequential step are indistinguishable and the tree is cheaper in both terms by the same factor. Had the conjecture stood, the tree bound would have traded one term against another and a scheduler would have needed a crossover analysis; it does not, and there is no crossover.

**Inference — the headline.** The bound is a function of the fold tree's *depth*, not of its contributor count:

```text
P(D, h2)  =  (1 + eps_exp)^D * (1 + gamma_{h2 + D}) / (1 + gamma_{h2})  -  1   ~=   D * (u + eps_exp)
```

where `D` is the **rescale depth** (merge levels on the deepest root-to-leaf path at which the partial states carry different maxima) and `h2` the summation depth of the two-pass fold over the *same* tree. The sequential form is the instance `D = h2 = V - 1`, at which this reduces **exactly** to the certified-bounds record's `P(V)` — checked as an exact rational equality against that record's own retained implementation at `V` in `{2, 8, 64, 512, 8192}`, and watched failing under two separate perturbations.

**Inference — the telescoping survives intact, with no weakened form to state.** Along any root path of any merge tree the subtree maxima are non-decreasing by nesting alone and the first already dominates the contributor, so the perturbations sum to `(m_V - x_j) * u` exactly as in the chain. The logit spread cancels between the two folds unchanged. The argument mentions no `V`, no height, no balance, and no branching, so the unbalanced case needs no separate statement: `D`, `N`, and `h2` are maxima over root paths and a ragged tree takes its own.

**Inference — tighter, not looser, and by a large factor.** At `V = 512`: `6.09e-5` sequential against `1.07e-6` for a pairwise merge tree and `3.58e-7` to `5.96e-7` for block sizes 64/32/16 — 34x (flash-realistic sequential outer loop) to 170x tighter. At `V = 8192`, 130x to 1366x. **The block size does not appear in the price; the block *count* does**, so the ambiguity in whether "B" means block size or count dissolves rather than needing a ruling — `16x32` and `32x16` are both measured. The intra-block fold cancels entirely, which the `v512-block64-serial-intra` shape shows at the extreme: a 63-deep serial prefix raises `h2` from 9 to 66 and leaves the price at `D = 3` unchanged to six digits.

**Inference — the shape parameter a schedule must hand the admission rule is the pair `(D, h2)`**, both combinatorial properties of the scheduled fold tree, both computable at compile time from schedule and target profile with no input value. `V` alone is neither sufficient nor necessary: two schedules over the same 512 contributors differ by 170x, and two different contributor partitions give the same price. This satisfies **the third clause and only the third clause** of ADR 0095's second reopening condition; the condition has not fired, because no rule exists in the certified-bounds admission shape and `eps_exp` is still not retrievable from the target authority. Nothing here reopens or relitigates the reaffirmed decline.

**Inference — a third gate the reopening analysis did not carry.** A *tree*-shaped rescaling fold consumes **reassociation** in addition to distributivity and the elementary identity, because reaching a tree grouping from the pinned strict left fold is a regroup neither of the other two performs. Reassociation is already grantable, so this is a conjunct to check rather than a third blocked door — but a joint decision that priced only two dimensions would be pricing the sequential fold while describing the parallel one. The sequential form is the boundary case that confirms this: Algorithm 3 *is* the left-deep merge tree, whose expansion is the canonical grouping, so `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` naming exactly two freedoms is correct as written and this is an addition rather than a correction.

**Inference — one distinction found and reported rather than smoothed.** `P` is the ratio of the two folds' *bounds*, so it bounds the extra budget the rewrite consumes; it does **not** bound the realized divergence `|online - two_pass| / R`, whose rigorous bound is `B_online + B_2`, about twice `P`. The retained probe checks the divergence against `P`, which is a good detector and a bad theorem. Nothing in the admission rule depends on the distinction — that rule compares a rewrite's bound against a tolerance — but the wording needs one sentence, and [`separate-the-rescaling-price-from-the-observed-fold-divergence`](separate-the-rescaling-price-from-the-observed-fold-divergence.md) owns it because that record's derivations and `contracts/numerics` were outside this ticket's scopes.

**Measurement.** 91 rows — 13 declared fold shapes crossed with the 14 logit sets of matching contributor count from the retained adversarial corpus — in exact binary32 semantics against a 120-digit reference, CPython 3.11.13 / arm64 / Darwin. No derived bound violated, no subnormal intermediate reached, both modes byte-identical (verified with `diff`). The left-deep merge tree reproduces the retained probe's online and two-pass folds **bit for bit** on all 14 of its rows. Exact invocation, from the repository root: `python3 spikes/numerics/online_softmax_tree_bound_probe.py` and `python3 -O spikes/numerics/online_softmax_tree_bound_probe.py`.

**Six perturbations watched failing**, in a scratch copy of both probe files, unperturbed copy returning exit 0 before and after each: flipped rescale sign (175 failures over 91 rows, including 10 bit-exactness); price divided by 100 (25, of which all 5 specialization mismatches); shape deleted (both population literals); price exponent `E-1` to `E`, which loosens the price so only the specialization cross-check can catch it (5); a spread of 95 driving `exp` into the subnormal band (4 rows plus both population literals); and `ceil(log2 blocks)` substituted for the merge tree's actual rescale depth.

**The last one is the honest limitation and is reported as the finding it is: it failed on 1 row of 91.** Understating `D` by 56x leaves 90 rows passing because the bound is loose by one to nineteen orders of magnitude over this corpus. **This population refutes structural errors in the fold; it does not pin the shape parameter.** The counting argument is carried by the derivation and by the exact specialization to the already-published sequential case, not by these numbers.

**Files.** New: `docs/research/numerics/tree-fold-online-softmax-bound.md`, `spikes/numerics/online_softmax_tree_bound_probe.py`, `spikes/numerics/online_softmax_tree_bound/README.md`, `spikes/numerics/online_softmax_tree_bound/results.json`. The new probe **imports** the retained one rather than copying it, so the two cannot drift and the two cross-checks above are possible; the retained probe and its `results.json` provenance are byte-untouched, which is why this is a second spike directory rather than a rewrite of the first. `docs/research/numerics/certified-bounds-as-rewrite-permissions.md` receives **one dated pointer only**, appended to the open axis this record closes; nothing else in that record is edited, and its Part 3 obligation 3 sentence — "A bound derived for a sequential fold does not admit a tree fold, and the derivation above is explicitly the sequential case" — was checked and is still true after this work, so it needed no edit.

**Filed rather than absorbed.** [`separate-the-rescaling-price-from-the-observed-fold-divergence`](separate-the-rescaling-price-from-the-observed-fold-divergence.md) (todo); [`carry-the-rescale-depth-a-parametric-bound-instantiates-from`](carry-the-rescale-depth-a-parametric-bound-instantiates-from.md) (deferred, with trigger check log); [`measure-whether-a-targets-exponential-is-exact-at-zero`](measure-whether-a-targets-exponential-is-exact-at-zero.md) (deferred, with trigger check log).

**No gate input was touched** — nothing under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh` — so no cargo gate was run. `tkt lint` and `git diff --check` were run fresh.

### Catalog rows for the coordinator, verbatim

`contracts/foundation` owns both catalogs and this ticket does not hold it, so the rows are recorded here rather than written. **Every link inside the two fenced blocks below is written relative to the row's destination file and does not resolve from this ticket** — that is the point, and the rows are transferred verbatim rather than repointed, because repointing would break the identity the transfer exists to preserve. Both were checked against their destinations: all five targets exist relative to `docs/research/README.md` and `spikes/README.md` respectively.

**`docs/research/README.md`**, at the end of the `### Numerical operations` list — it sorts after `Transformer non-linear, normalization, and reduction contracts`:

```text
- [The tree-fold form of the online-softmax rescaling bound](numerics/tree-fold-online-softmax-bound.md) — pending; sound-proof, bounded-measurement; informs: [Numerical semantics](../numerical-semantics.md), [Optimizer model](../compiler/optimizer.md); experiments: [Tree-fold online-softmax bound probe](../../spikes/numerics/online_softmax_tree_bound/README.md)
```

**`spikes/README.md`**, at the end of the `### Numerical operations` list — it sorts after `Transformer reference-semantics probe`:

```text
- [Tree-fold online-softmax bound probe](numerics/online_softmax_tree_bound/README.md) — reproducible; bounded-measurement; supports: [The tree-fold form of the online-softmax rescaling bound](../docs/research/numerics/tree-fold-online-softmax-bound.md)
```

**Not proposed, flagged instead.** `docs/numerical-semantics.md`'s `evidence:` frontmatter lists research ids and already carries `tiler.research.numerics.certified-bounds-as-rewrite-permissions` at `pending` disposition, so `tiler.research.numerics.tree-fold-online-softmax-bound` would be consistent there. It is `contracts/numerics`, it changes no contract sentence, and adding an evidence id for a record nothing normative rests on is a judgement this ticket declines to make on the owner's behalf.

## Four-outcome disposition

**Closes.** The tree-fold bound is derived, the telescoping re-established rather than assumed, the unbalanced case covered by construction, the sequential comparison quantified at the shapes attention uses, and the schedule-side shape parameter named and shown compile-time computable. The certified-bounds record's "the tree-fold form of the bound is underived" axis is answered.

**Parks.** The price-versus-divergence wording, the schedule field that would carry `D`, and the inherited `eps_exp` retrieval gap — each on a filed ticket, none absorbed. No contract sentence moved and no ADR was edited.

**Experiment remaining.** One bounded measurement: whether a target's `exp` is exact at a zero argument, which halves the elementary term and changes the constant rather than the form. Metal's `exp <= 4 ulp` does not imply it, so the conservative charge stands until measured.

**Reconsideration trigger.** ADR 0095's second reopening condition, whose third clause this satisfies and whose other two it does not. Reproduce the verdict with `grep -n 'Second reopening condition' docs/decisions/0095-decline-a-distributivity-permission.md` and `tkt show expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`.
