---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.certified-bounds-as-rewrite-permissions"
kind: "research"
title: "Certified rounding-error bounds as rewrite permissions"
topics: ["numerics", "accuracy", "proof", "optimizer", "reductions"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "sound-proof", "bounded-measurement"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.optimizer"]
depends_on: ["tiler.research.numerics.region-accuracy-contract", "tiler.research.numerics.sound-region-analyzer-spike", "tiler.research.region-search.rewrite-search-formalism"]
ticket: "connect-certified-rounding-error-bounds-to-rewrite-permissions"
---

# Certified rounding-error bounds as rewrite permissions

**Status:** survey complete; the admission rule's shape is derived and the worked bound is proved and measured. Nothing here is a contract change, and the tolerance vocabulary below is a proposal identified for Tom rather than designed into anything.

## Traceability

- **Current disposition:** pending. No ADR adopts this record and no contract sentence has moved for it.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md) would own a tolerance vocabulary; [the optimizer model](../../compiler/optimizer.md) would own where the admission rule sits. Neither is edited by this record, and `contracts/numerics` was outside the producing ticket's scopes.
- **Evidence:** the preserved fifth-wave sources under [`sources/`](sources/README.md), and the executable witness at [`spikes/numerics/online_softmax_bound/`](../../../spikes/numerics/online_softmax_bound/README.md).
- **Builds on:** [Region accuracy contracts and analyzable error budgets](region-accuracy-contract.md), which established the goal, reference, metric, and evidence-class vocabulary this record uses without restating; [the sound region-analyzer spike](sound-region-analyzer-spike.md), which measured the analyzer route this record eliminates from the online path; and [the rewrite-search formalism](../region-search/rewrite-search-formalism.md), which fixed the staged alternative-retaining search the rule has to sit inside.
- **Work record:** [`connect-certified-rounding-error-bounds-to-rewrite-permissions`](../../../tickets/connect-certified-rounding-error-bounds-to-rewrite-permissions.md).

## Outcome

**A rewrite's rounding cost can be a *derived parametric bound carried by the rule*, checked against a caller-stated tolerance by an exact rational comparison, answering `Admit` / `Refuse` / `Undecided` with `Undecided` fail-closed.** It cannot be a bound derived per instance by an external analyzer inside the search, and the elimination below is on measured cost and on a coverage gap rather than on preference.

Three results carry that conclusion.

**Fact — the classical analysis instantiates.** The online-softmax rescaling fold, which is the flash-class rewrite the attention work actually wants, has a closed-form worst-case relative error bound derived here from the standard model of floating-point arithmetic. Its price over the two-pass fold it replaces is at most `(1 + eps_exp)^(V-1) * (1 + gamma_{2(V-1)}) / (1 + gamma_{V-1}) - 1`, first-order `(V-1)(u + eps_exp)`, in the contributor count `V`, the format's unit roundoff `u`, and the target's declared relative accuracy for `exp`. **The logit spread cancels**: it appears in each fold's own bound and not in the difference between them, which is what makes the price instantiable from a shape and a target profile with no knowledge of any input value.

**Measurement — the bound holds and is loose by one to three orders of magnitude.** Over 22 named adversarial cases in exact binary32 semantics against a 120-digit reference, the derived bounds are never violated, and the ratio of bound to observed error runs from 4.8 at `V = 2` to 2.7e29 where every term but one underflows the sum. That looseness is not a defect in the derivation; it is what a worst-case bound over positive-term summation is, and it is the single most consequential fact for the admission rule, because it means the rule will refuse rewrites that would in fact have met the tolerance.

**Inference — the rewrite consumes a permission Tiler does not have, and it is distributivity.** The rescaling fold is not a reassociation: its contributor sequence is not a regrouping of the two-pass fold's contributors, because each partial sum is multiplied by a rescale factor. Expanding the nested form gives back the sum of products, so the identity consumed is exactly the one [ADR 0080](../../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) names and [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) declined to admit a permission for — and it consumes a second freedom the vocabulary names nowhere, the functional equation of the exponential. This does not fire ADR 0095's reopening trigger, which is written about contraction chains, and it does refute the premise that decline rested on. Both are filed rather than decided here.

## Part 1 — What the literature bounds, and what it refuses

Every claim in this part is read in the preserved copy named beside it. The [source record](sources/README.md) carries provenance, licence, and the acquisition failures; this part carries only what the documents say.

### The two analysis families, in the words of a tool that implements both

[Daisy's](sources/README.md#daisy-tacas-2018) command-line menu is the cleanest taxonomy in the corpus because it is a working classification rather than a survey's: `--analysis=[dataflow:opt:relative]`, where forward dataflow analysis is "as implemented in Rosa, Fluctuat and Gappa" and optimization-based analysis is "as implemented in FPTaylor and Real2Float". The five tools this survey was asked to cover therefore reduce to two mechanisms plus a search heuristic, which is why the parts below are organized that way rather than tool by tool.

**Forward dataflow analysis** propagates an interval or affine form through the expression, adding each operation's rounding term. It is compositional, fast, and pessimistic — its overestimates are what the optimization-based family exists to fix.

**Optimization-based analysis** — [FPTaylor's](sources/README.md#fptaylor-toplas-2018) symbolic Taylor expansions — treats the error as a function of the rounding variables and bounds it by rigorous global optimization "instead of the more familiar interval arithmetic, affine arithmetic, and/or SMT solvers", because those "often provide very pessimistic overestimates, causing unnecessary verification failure".

### What each tool bounds, cannot bound, and whether its answer is checkable

| Tool | Bounds | Refuses | Certificate | Preservation |
| --- | --- | --- | --- | --- |
| FPTaylor | absolute and relative roundoff for straight-line code, transcendentals, mixed precision, subnormals | conditionals (only "preliminary comparative results"), loops | **HOL Light proof, machine-checkable** — but "the general improved rounding model is not formally verified yet" | metadata-only (ACM) |
| Gappa | enclosures of expressions with rounded operators, by interval arithmetic and forward error analysis | needs user-supplied *hints* to succeed on hard goals | **Coq or HOL Light proof per enclosure**, on a companion library | metadata-only (ACM, via HAL) |
| Daisy | `+ - * / sqrt`, FMA, `sin cos tan log exp`, fixed and floating point, f16 through quad, mixed precision | "does not support conditionals and loops"; a violated postcondition **warns** rather than refuses | none — the analyzer's word | **vendored** (CC BY 4.0) |
| Real2Float | polynomial and transcendental roundoff by semidefinite programming | non-polynomial structure outside its relaxation | **sums-of-squares certificate checkable in Coq** | metadata-only (arXiv licence) |
| Herbie | nothing. It *estimates* accuracy from 256 random points per comparison | "cannot provide worst-case error bound guarantees", by its own introduction | none | metadata-only (ACM) |
| Precimonious / HiFPTuner | nothing. Search over precision assignments validated by running the program | "No guarantees are made for other inputs" | none | metadata-only (ACM) |

**The table's rightmost distinction is the one that matters and it is not soundness.** Four of the six are sound; the split that decides Tiler's trust boundary is whether the answer is a *checkable artifact* or *an executable's assertion*. [Real2Float](sources/README.md#real2float-arxiv-150703331v7) counted the field at its 2016 writing: it names FPTaylor and itself as "the only academic software tools that can produce formal proof certificates", and observes that Rosa produces output "allowing independent soundness checking" but "does not formally verify these certificates". Gappa's Coq output predates both under a looser reading of "certificate", which this record notes rather than reconciles.

**Herbie and the tuners are in this survey to be placed, not to be adopted, and placing them is a real result.** Both are search procedures whose acceptance test is *sampling*. That is the `Empirical` class [the region-accuracy contract](region-accuracy-contract.md) already defines: samples can refute a claimed universal bound by exhibiting a counterexample and can never establish one. Herbie is therefore a legitimate *generator* of stage-3 semantic alternatives and never an admission authority — a distinction the tool's popularity makes easy to lose. Precimonious's own paper states the boundary in one sentence, that its analysis holds for the given "program inputs. No guarantees are made for other inputs."

### The one paper that already built what this ticket asks about, and put the bound in the wrong place

[Anton](sources/README.md#anton-rewriting-arxiv-170702118v1) (Darulova, Horn and Sharma, 2017) is "the first fully automated and sound technique and tool for optimizing the performance of floating-point and fixed-point arithmetic kernels", combining rewriting with mixed-precision tuning. Its rewriting "uses a genetic algorithm to search the vast space of possible evaluation orders", applying "real-valued identities, such as associativity and distributivity", and — the sentence this record turns on — "the search is guided by a fitness function which bounds the roundoff errors for a candidate expression - the smaller the error, the better". Daisy's own `--rewrite` mode does the same: "genetic search to find a rewriting for which it can show the smallest roundoff error".

**A sound worst-case bound used as a search objective is precisely the placement [Tiler's optimizer contract](../../compiler/optimizer.md#the-four-surfaces-the-optimizer-may-consult) forbids.** That contract holds hard feasibility separate from estimated cost in both directions, and prohibits pruning any semantic alternative on an estimate at any stage. Anton is evidence that the alternative design is real, published, and adequate for its own problem — scalar straight-line kernels with one output and one caller — rather than a straw man. It is also evidence for why Tiler cannot copy it: a fitness function ranks, and ranking cannot express "this caller stated 2^-16 and this rewrite costs 2^-13, so it is not available at any price".

[FPTuner](sources/README.md#fptuner-popl-2017) is the corpus's one instance of the other shape, and therefore the closest prior art to an admission rule: it "generates and solves a quadratically constrained quadratic program", carrying the error requirement as a **constraint** with a separate objective. That is structurally what this record proposes, arrived at independently from Tiler's own contract.

### The classical analysis, and the one document nobody here has read

The worked example needs three results, all read in [the Acta Numerica survey](sources/README.md#acta-numerica-fp-2023) (Boldo, Jeannerod, Melquiond and Muller, 2023), which is vendored under CC BY:

- **(2.5a), the standard model.** `RN(a op b) = (a op b)(1 + eps_1)`, `|eps_1| <= u`, for `op` in `{+, -, *, /}`, absent overflow and absent a subnormal result — and the survey is explicit that "if the result of an operation is subnormal, one cannot in general guarantee a relative error bound such as (2.5a)", the absolute error being bounded by `2^(emin-p)` instead.
- **(4.7), the `gamma` bound.** With `1 + theta_h` a product of `h` such factors, `|theta_h| <= h*u/(1 - h*u) =: gamma_h` for `h < 1/u`, composing by `gamma_h + gamma_k + gamma_h*gamma_k <= gamma_{h+k}`.
- **(4.8), the summation bound.** For `s_n` computed with `n-1` additions "and any parenthesizing", the backward error is `gamma_h` where "`h <= n - 1` is the height of the binary tree that underlies the chosen summation algorithm", giving `|s_n_hat - s_n| <= gamma_{n-1} * sum |x_i|` **independently of ordering**, with `h = n-1` recursive, `h = ceil(log2 n)` pairwise, and `h = b + n/b - 2` blocked.

**A tighter constant exists and is deliberately not used.** [Jeannerod and Rump](sources/README.md#jeannerod-rump-simax-2013) replace `gamma_n` by `n*u` for round-to-nearest, removing the second-order term and the `n*u < 1` side condition; Lange and Rump give `(n-1)*2u` for faithful rounding. The derivation below keeps `gamma` because `gamma`'s composition rule is what lets the fold's bound compose with the elementary function's in one algebra, and a mixed derivation would have to justify each join separately. Tightening it is a filed follow-up, not an oversight.

**The survey attributes all three results to Higham's *Accuracy and Stability of Numerical Algorithms* (SIAM, 2002) at §3.4 and §4.2, and this project has not read that book.** It sits behind a purchase wall — `https://epubs.siam.org/doi/book/10.1137/1.9780898718027` returns HTTP 403 to a non-browser client and the SIAM catalogue page does the same, and no open-access, arXiv, or repository edition exists to locate. It is the manifest's one `pending-acquisition` row and it is an acquisition request for Tom. **What it would and would not decide is stated at that row rather than here**, and the short form is: no claim in this record is deferred behind it, because the three results are read in a peer-reviewed restatement by four authors of the field; what acquiring it would supply is the *proofs* rather than the statements, and the ability to check that the restatement lost no side condition. Reading a foundation at one remove is the honest description of this record's position and it is stated rather than glossed.

## Part 2 — The worked example: the online-softmax rescaling bound

This part is the sanity anchor. It is worked in full because a survey that selects a tool without ever deriving one bound by hand cannot tell a tight answer from a plausible one.

### The rewrite

[Milakov and Gimelshein's](sources/README.md#online-softmax-arxiv-180502867v2) Algorithm 2 is the safe softmax every framework uses, computing the normalizer in two passes over the logits: `m_V = max_k x_k`, then `d_V = sum_j exp(x_j - m_V)`. Their Algorithm 3 fuses the two into one pass:

```text
m_j <- max(m_{j-1}, x_j)
d_j <- d_{j-1} * exp(m_{j-1} - m_j) + exp(x_j - m_j)
```

Their Theorem 1 proves by induction that this computes the same `m_V` and `d_V`. **That proof is over the reals.** The paper makes no floating-point claim about the recurrence anywhere, and its measurements are throughput. The gap that leaves is exactly what a compiler needs before it may perform the rewrite.

### What permission the rewrite consumes — and the finding that it is not reassociation

Unrolling the recurrence in exact arithmetic gives `d_V = sum_j [ exp(x_j - m_j) * prod_{k>j} exp(m_{k-1} - m_k) ]`, and the exponentials telescope: `exp(x_j - m_j) * exp(m_j - m_V) = exp(x_j - m_V)`, which is the two-pass contributor. So the two forms agree over the reals, as Theorem 1 says.

**But the fold is a Horner nesting, not a re-parenthesized sum.** Reaching it from `sum_j t_j` requires distributing each rescale factor over a partial sum — `(a + b) * r = a*r + b*r` — which is [ADR 0080's](../../decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) dimension exactly: exchanging a product of a sum for a sum of products. The contributor sequences share no floating-point value, so no reassociation permission and no permutation permission reaches this rewrite. Under [ADR 0011](../../decisions/0011-per-operation-numerical-permissions.md), which requires every rewrite to declare the permission it consumes and holds that one permission never implies another, **the online-softmax rescaling is currently illegal under every numerical contract Tiler can express** — not unimplemented, illegal, and for a reason no existing document states.

**A second freedom is consumed and Tiler's vocabulary does not name it at all.** The telescoping step uses `exp(a) * exp(b) = exp(a + b)`, a functional equation of the elementary function rather than an algebraic identity of the ring — and in floating point it is false: `fl(exp(a)) * fl(exp(b))` and `fl(exp(a + b))` differ. The order-contract dimensions govern how contributors are combined; nothing governs rewriting *through* a transcendental's identities. This is a gap in the dimension set rather than a missing permission within it, and it is filed.

**This does not fire ADR 0095's reopening trigger, and stating why is what keeps the record honest.** That trigger is "the first workload whose natural spelling is a directly regroupable *contraction chain*", and the softmax normalizer is an elementwise-scaled reduction, not a contraction chain; a candidate that cannot be distinguished from the worked negative case does not fire it either, and this one is distinguishable in the wrong direction — it is not that shape at all. **What it does do is refute the premise the decline rested on.** ADR 0095's ground was that "No caller exists" and that admitting a permission nobody can spend adds a dimension to every contract for nothing. A caller now exists for the *dimension*: the flash-class one-pass softmax is a rewrite the attention work wants, whose profitable form requires the distributive exchange. Whether that reopens the decision is Tom's, on evidence including the price derived below; this record files the question and decides nothing.

### The derivation

**Assumptions, each of which the admission rule must discharge rather than inherit.** Binary32, round-to-nearest-ties-to-even, unit roundoff `u = 2^-24`. No overflow. No subnormal intermediate, because (2.5a) does not hold there. `max` is exact, as a selection among representable values. The `exp` routine has relative error at most `eps_exp`, taken from the target's declared realization rather than assumed. The contributor count `V` satisfies `2(V-1)*u < 1`.

**Step 1 — each term's error, and why the spread appears.** Let `a_j := x_j - m_V <= 0` as exact reals. The two-pass form computes `fl(x_j - m_V) = a_j(1 + delta)`, `|delta| <= u`, so the *argument* carries one rounding, and `exp` turns an argument perturbation into a relative one by its own magnitude: `exp(a_j(1+delta)) = exp(a_j) * exp(a_j*delta)`, a factor bounded by `exp(A*u)` where `A := max_j |x_j - m_V|` is the **logit spread**. Applying `exp` then adds `(1 + eps_exp)`. So each two-pass term carries a relative factor bounded by `(1 + eps_exp) * exp(A*u)`.

This is worth pausing on: **the softmax term error is governed by the logit spread, not by the contributor count.** A workload with widely separated logits pays for that separation in both folds, which is why the spread must not be allowed to hide inside a bound that is presented as a function of shape alone.

**Step 2 — the two-pass fold.** Every term is positive, so `sum |t_j| = sum t_j = R`, the reference, and (4.8) applies with the fold's own tree height. For recursive summation of `V` terms that height is `V-1`:

```text
B_2  =  (1 + eps_exp) * exp(A*u) * (1 + gamma_{V-1})  -  1
```

**Step 3 — the online fold, counting what the earliest term passes through.** Term `j` enters at step `j` and is then acted on by every later step. Each later step applies one multiply (by the rescale factor) and one add, where the two-pass fold applies one add. So term 1, the worst case, passes through `2(V-1)` roundings rather than `V-1`. It also passes through `V` calls to `exp`: its own, plus one rescale factor per later step.

**The argument perturbations are the interesting part, and they telescope.** Term `j` carries the rounding of `fl(x_j - m_j)`, magnitude `|c_j| = m_j - x_j`, plus the rounding of each later `fl(m_{k-1} - m_k)`, magnitude `|b_k| = m_k - m_{k-1}`. The running maximum is non-decreasing, so those sum exactly:

```text
|c_j|  +  sum_{k>j} |b_k|   =   (m_j - x_j) + (m_V - m_j)   =   m_V - x_j   =   |a_j|   <=   A
```

**The online fold's argument-perturbation factor is therefore identical to the two-pass fold's.** The rescaling does not accumulate spread; it redistributes it. This is the derivation's one non-obvious step and it is what makes the rewrite affordable at all — a naive count would have given `V*A*u` and made the rewrite look far worse than it is.

```text
B_1  =  (1 + eps_exp)^V * exp(A*u) * (1 + gamma_{2(V-1)})  -  1
```

**Step 4 — the price, which is the quantity a caller's tolerance is compared against.** Dividing out the factor common to both folds:

```text
P(V)  =  (1 + eps_exp)^(V-1) * (1 + gamma_{2(V-1)}) / (1 + gamma_{V-1})  -  1
      ~=  (V - 1) * (u + eps_exp)                          to first order
```

**`exp(A*u)` is absent, and that absence is the load-bearing result.** The price of the rewrite is a function of the contributor count, the format, and the target's elementary accuracy — and of no input value. It can therefore be instantiated at compile time from a shape and a target profile, which is exactly what an admission rule inside a search needs and exactly what a per-instance analyzer cannot deliver cheaply.

**Worked instantiation.** At `V = 512`, binary32, `eps_exp = u` (a correctly rounded `exp`): `P = 6.09e-5`, about `2^-14`. A caller stating a `2^-13` relative tolerance admits the rewrite; a caller stating `2^-16` refuses it. That is the quantitative permission this ticket set out to make possible, in one number a caller can price.

### Evidence class, stated precisely because three classes are in play

The derivation is **`sound-proof`** in [the metadata contract's](../../document-metadata.md#kinds-and-status-facets) sense — the property follows within the documented model and its stated assumptions — and it is a pencil derivation that no machine has checked. Its foundation is read in a vendored survey rather than in the monograph that proves it. Its instantiation at `eps_exp = u` is a *choice* about the target and not a fact about any target.

The **`bounded-measurement`** half is [the probe](../../../spikes/numerics/online_softmax_bound/README.md), which evaluates both folds in exact binary32 semantics against a 120-digit reference over 22 named adversarial cases and checks all three bounds. It found no violation. **Agreement over a finite corpus does not prove a universal bound**, and the probe's own header says so; what it adds is the looseness, which the derivation alone cannot supply.

### Measurement — how loose the bound is, and why that decides the design

Bound-over-observed ratios from [`results.json`](../../../spikes/numerics/online_softmax_bound/results.json), on CPython 3.11.13 / arm64 / Darwin:

| Case | `V` | spread `A` | two-pass bound / observed | online bound / observed | observed price | derived price |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `increasing-v2-span1` | 2 | 1 | 4.85 | 8.08 | 0 | 1.19e-7 |
| `increasing-v8-span1` | 8 | 1 | 44.3 | 16.9 | 9.32e-8 | 8.34e-7 |
| `increasing-v64-span1` | 64 | 1 | 88.2 | 18.5 | 6.59e-7 | 7.51e-6 |
| `increasing-v512-span1` | 512 | 1 | 59.3 | 102 | 1.41e-6 | 6.09e-5 |
| `sawtooth-v512` | 512 | 7 | 67.7 | 61.4 | 1.04e-6 | 6.09e-5 |
| `decreasing-v512` | 512 | 40 | 110 | 312 | 0 | 6.09e-5 |
| `uniform-v512` | 512 | 0 | observed zero | observed zero | 0 | 6.09e-5 |
| `dominant-tail-v512` | 512 | 60 | 7.6e18 | 2.1e19 | 0 | 6.09e-5 |

**Three findings, in descending order of consequence for the admission rule.**

**The bound is loose by one to two orders of magnitude in the ordinary regime, and that is intrinsic rather than fixable by a better derivation.** `gamma_{V-1}` is a worst case attained when every rounding goes the same way; a sum of positive terms of comparable magnitude does not approach it. **The consequence is that a sound admission rule will refuse rewrites that would have met the caller's tolerance**, by roughly the ratio above. That is the correct failure direction and it must be stated in the contract that offers the vocabulary, because a caller who sets a tolerance from measured error and finds the rewrite refused will otherwise conclude the compiler is broken.

**The price is zero whenever the running maximum stops moving.** `decreasing`, `uniform`, and `dominant-tail` all show an observed price of exactly zero: once `m_j` is fixed, every rescale factor is exactly `1`, and the online fold degenerates into the two-pass fold. The derived price does not know this. A shape-parametric bound that could consult a *proved* value precondition — "the logits are non-increasing" — would collapse to zero here, which is a concrete case where the region-accuracy contract's relational assumption machinery would pay for itself.

**Looseness grows without limit when one term dominates.** In `dominant-tail`, the reference is within rounding of a single term and the summation of `511` negligible terms is nearly exact, so the ratio reaches `10^19`. This is the regime attention softmax is usually in, and it means the bound is at its least useful exactly where the workload lives.

### Proving the probe can say no

Four perturbations were run against the committed probe in a scratch copy; the exact commands and outputs are in [the spike record](../../../spikes/numerics/online_softmax_bound/README.md). Flipping the sign of the rescale argument — a real defect in the rewrite itself — produces 32 failures over 22 cases, 16 of each kind. Understating the price by two orders of magnitude produces 5 failures, against 7 cases whose observed price is non-zero; the two it does not catch are the two whose price is smallest relative to the derived one, which is the expected boundary rather than a gap in the check. Deleting a declared case fails the population count. Driving `gamma` past `h*u = 1` raises rather than returning a vacuous negative bound.

**One of the four found a defect in the probe rather than confirming it, and that is why the run was worth doing.** The population check originally compared `len(rows)` against `len(corpus())` — both derived from the same function, so deleting a case left the check agreeing with itself and exiting zero. It now compares against a literal `DECLARED_CASES = 22`, the same discipline `verify-sources.sh` states at its own top. A check whose two sides come from one source cannot say no.

**What the corpus cannot detect, stated so the evidence is not overread.** Removing the elementary-function factor from the online bound, or removing its summation term entirely, both still pass — the remaining headroom is large enough to absorb a missing first-order term. This corpus refutes structural errors in the fold and gross understatements of the price; **it does not pin the bound's constants**, and no reading of `results.json` should suggest it does.

## Part 3 — The admission rule

### The elimination: per-rule parametric bound versus per-instance derivation

Two candidates were tested against correctness, performance, and long-term maintainability, per the elimination discipline. A third is the hybrid.

**Candidate A — per-instance derivation.** At compile time, hand each scheduled candidate to an external analyzer and take its bound.

**Candidate B — per-rule parametric bound.** Each rewrite rule ships a closed-form bound, derived and reviewed once, parametric in shape and target facts; the compiler instantiates and compares it.

**Candidate C — B as the admission rule, A as an optional tightener.**

**Correctness eliminates neither, and pretending otherwise would be the easy mistake.** Both are sound if their obligations are discharged. A carries the analyzer's trusted computing base, which [the sound region-analyzer spike](sound-region-analyzer-spike.md) measured and described as "a materially larger trusted computing base than a small certificate checker" — and, more tellingly, that spike could not reproduce its own historical proof streams, because the executable closure was not retained. B carries a reviewed derivation plus an instantiation check, both in-tree and both testable. Neither is disqualified; A is simply more expensive to trust.

**Performance eliminates A as the online rule, decisively.** The same spike measured 8–320 ms of analysis with roughly 1.0–1.5 s total invocation per profile, dominated by Scala frontend startup. [The optimizer's search](../../compiler/optimizer.md) retains *every* legal alternative at stage 3 and may not prune any of them on an estimate, so the number of analyzer invocations is the size of the alternative set — and the alternative set is precisely what a flash-class rewrite vocabulary is meant to grow. A second per candidate is not a cost to optimize; it is a different compiler. Instantiating a closed form, by contrast, is exact-rational arithmetic on the `ExactRational` machinery `tiler-reference` already carries.

**Coverage eliminates A a second time, for the first rewrite that needs it.** The analyzer profile that spike admitted covers `+ - * / sqrt` and required FMA, and states that "transcendentals other than the specifically modeled real square root remain outside the profile". The softmax rewrite is `exp` throughout. **This is an adapter gap rather than a Daisy gap** — Daisy's own tool paper lists "the standard transcendental functions (sin, cos, tan, log, exp)" — and the correction matters, because it changes the remedy from "wait for the field" to "extend the profile". Either way, A cannot bound the first rewrite anyone wants today.

**Maintainability favours B and the margin is not close.** A couples the compiler to an external Scala or OCaml toolchain, a JDK, and an SMT solver; the spike recorded current Daisy failing to compile on the installed JDK 8, 17, and 26. B keeps the obligation in-tree beside the rule, which is where [ADR 0011](../../decisions/0011-per-operation-numerical-permissions.md) already requires the rule to declare the permission it consumes.

**Result: B survives; A is eliminated as the admission rule and retained in a different role.** C is B plus an optional path with no caller, so it is B plus a deferred question — and the honest synthesis is that the analyzers keep a real job. **A per-rule bound has to be derived by somebody, and an analyzer is exactly the right tool for cross-checking a derivation on a bounded corpus offline.** That is where FPTaylor, Gappa, Daisy, and Real2Float belong in Tiler: on the bench beside a derivation, not in the compiler beside the search. This record's own derivation was cross-checked against a probe rather than against an analyzer, which is the weaker of the two available checks and is named as such.

**What would refute this elimination.** An analyzer that answers in microseconds from a warm in-process worker, over a profile covering `exp`, with a certificate a small in-tree checker validates. Nothing in the corpus is close, and the trigger is filed rather than left as an impression.

### Where the rule sits, and the one thing it must never become

The rule is a **feasibility answer at stage 3 semantic exploration**, asked per candidate, in the same position and with the same shape as the elementary-accuracy obligation the compiler already asks there. That precedent is exact and worth naming: `request::require_elementary_accuracy` calls `target::accuracy::assess_program_elementary_accuracy`, `readmit_candidate` asks it again of every semantic candidate so that "a rewrite cannot inherit an admission granted to a program that did not contain the family", and the authority is conservative in one direction only — it may reject a legal implementation and can never admit an illegal one. A rewrite-bound admission is the same kind of object over a different subject.

**It must never become a cost.** [The optimizer contract's](../../compiler/optimizer.md#the-four-surfaces-the-optimizer-may-consult) fourth-surface prohibition is the invariant: hard feasibility is never expressed as a cost in either direction, and no semantic alternative is pruned on an estimate at any stage. A bound that ranked admitted alternatives would reintroduce exactly the cost-pruning of semantic alternatives the [rewrite-search formalism](../region-search/rewrite-search-formalism.md) eliminated, and would do it wearing a soundness argument — which is the hardest form to notice in review. **Anton and Daisy's `--rewrite` both make that choice**, so the failure mode is not hypothetical and the record names published instances of it rather than describing a risk.

Among *admitted* candidates the ordinary cost model still trades execution time, memory, and compile cost, exactly as [the region-accuracy contract](region-accuracy-contract.md) already states for analyzer evidence: "The analyzer returns feasibility evidence, never a cost."

### The trust boundary — what validates a certificate, and what refuses one

The decision is three-way, and **this shape already exists in the tree**: `tiler_reference::accuracy::decide_predicate` answers `Conforms` / `Violates` / `Undecided`, and `ConformanceDecision::conforms()` returns `false` for both `Violates` and `Undecided` — the fail-closed reading, documented there as the alternative to "a check that cannot fail, which is the failure mode this repository distrusts most". The admission rule is the same three-way exact-rational comparison applied to a rewrite's bound and a caller's tolerance, and it should reuse that machinery rather than grow a parallel one.

**Five validation obligations, each of which can refuse.**

1. **The bound's side conditions are discharged, never inherited.** The contributor count is bounded; no intermediate is subnormal; no overflow occurs; `2(V-1)*u < 1`. Every one of these is a condition (2.5a) or (4.7) states and each is a real refusal — a subnormal intermediate voids the relative-error model outright, and the derivation is silent rather than conservative there.
2. **The `eps_exp` instantiation comes from the target, not from a constant.** This is where the proposal meets a gap in the tree. `assess_program_elementary_accuracy` answers whether a target's declared realization *provably refines* an operation's accuracy obligation — a yes-or-no refinement question. The bound needs the *numeric* relative accuracy to instantiate. That quantity is not currently retrievable from the authority that owns it, and the gap is filed.
3. **The evidence identity binds the complete scheduled candidate.** Fold height, contraction, intermediate formats, target numerical profile, and the assumption set — per the region-accuracy contract's rule that "A proof about the logical graph cannot qualify an independently changed physical plan". A bound derived for a sequential fold does not admit a tree fold, and the derivation above is explicitly the sequential case.
4. **The comparison is exact-rational, never a host float.** Tolerances are exact rationals or integers, as the region-accuracy contract already requires, and `gamma_h` is a rational function of `h` and `u` computable exactly. The probe does this and it is not incidental: a bound compared in binary32 would put a rounding inside the constant that bounds roundings.
5. **A value precondition is proved or transactionally validated before routing commit**, per [ADR 0021](../../decisions/0021-validated-value-assumptions.md), if the bound consults one. The `decreasing` and `uniform` measurements above show what such a precondition would buy: the price collapses to exactly zero when the maximum is known not to move.

**What refuses an unverifiable certificate.** Any obligation that cannot be discharged yields `Undecided`, which is not an admission. The candidate is refused with a named reason and **the baseline alternative is retained regardless**, because stage 3 always preserves it — so a refusal costs a candidate and never costs correctness. This is the property that makes a conservative rule affordable: the compiler that refuses the online-softmax rewrite still compiles the two-pass softmax.

**What must never be admitted.** A bound asserted by a rule with no derivation on file; a bound whose side conditions were checked against a different candidate; an analyzer's exit status in place of a parsed result (the spike found Daisy returning process status zero alongside an `OverflowException` diagnostic); a sampled maximum in place of a bound; and — the one a reviewer is likeliest to wave through — a bound whose `eps_exp` was filled in with a plausible constant because the target authority could not answer.

## Part 4 — Tolerance vocabulary: a proposal, not a contract change

**Nothing in this part is proposed for self-acceptance.** [Numerical semantics](../../numerical-semantics.md) is the normative owner, `contracts/numerics` was outside the producing ticket's scopes, and each item below is a public boundary under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md). This follows the [BF16 record's](bf16-computation-accumulator-and-conversion.md) precedent exactly: identify what a vocabulary addition would have to answer, and leave the spelling to Tom at implementation time.

**The additive shape, and why it breaks no existing key.** A numerical contract today resolves categorical permissions dimension by dimension — reassociation, permutation, contraction. A tolerance is not a fourth permission of that kind; it is a *budget scoped to a named rewrite family*, and the two compose without either changing. A contract that states no tolerance behaves exactly as today, which is the property that makes this additive rather than a migration.

The closest published spelling is [Rosa's](sources/README.md#rosa-toplas-2017), where an error clause is first-class in the grammar — `A ::= C | x +/- const | S` — so an uncertainty sits beside an ordinary constraint rather than annotating one, with `require(... && x +/- 1e-11)` on inputs and `ensuring(res => res +/- 1.001e-11)` on the result. [Daisy](sources/README.md#daisy-tacas-2018) uses the same `+/-` form. Both govern one scalar function's result; Tiler's observable is a named output of a multi-output graph, so the selector machinery the region-accuracy contract already defines is what the clause would attach to.

**Six items a vocabulary addition must answer, identified for Tom.**

1. **Whether a tolerance is a new dimension of the numerical contract or a separate goal attached beside it.** The region-accuracy contract already models the latter as `RegionAccuracyGoal` with `delegated_permissions` deliberately empty. Admitting rewrite tolerances is precisely the delegation that record deferred, and reusing its type rather than adding a contract dimension is the shape this record favours — but it is a public boundary and the two differ in what a caller writes.
2. **How a tolerance names the rewrites it governs.** The region-accuracy contract is explicit that a broad output tolerance must not become "an ambient `fast` flag", so a tolerance names the operations or numerical dimensions it delegates. For the worked example that means naming the softmax normalizer's fold, not the program.
3. **Whether the distributivity permission is admitted at all**, since the worked rewrite consumes it and [ADR 0095](../../decisions/0095-decline-a-distributivity-permission.md) declined one. A tolerance cannot authorize a dimension no permission grants; the two are independent gates and both must open.
4. **How the transcendental-identity freedom is named**, since it is a dimension the vocabulary does not have rather than a permission it lacks.
5. **What the refusal says.** ADR 0095 already requires a rewrite consuming distributivity to reject naming the missing dimension rather than reporting a forbidden reassociation. A tolerance refusal needs the analogous discipline: the stated tolerance, the derived price, and which of the five obligations failed — because "refused" without the number is unactionable when the number is the whole point.
6. **Whether the tolerance is stated as an absolute, relative, ULP, or mixed metric**, and against which reference. The region-accuracy contract enumerates the metrics and requires exact rationals; the worked bound above is *relative to a governed real lift*, and a caller stating a ULP tolerance would need a conversion the record does not derive.

## Part 5 — The FPTaylor deferral's trigger

[`spike-hermetic-fptaylor-certificate-checking`](../../../tickets/spike-hermetic-fptaylor-certificate-checking.md) is `deferred` behind a conjunction: the bounded analyzer integration complete, **and** a milestone requiring independently checkable accuracy evidence rather than the accepted trusted-analyzer result, **and** a pinned hermetic toolchain plan with explicit approval for any host installation.

**Verdict: not fired, and this record moved the second clause further away rather than closer.** The trigger log on that ticket carries the dated entry. The reasoning: the admission rule this record derives is a per-rule bound whose trusted base is a reviewed in-tree derivation plus an exact-rational instantiation check. **It routes through no analyzer at all**, so it needs neither a trusted-analyzer result nor an independent certificate to replace one. The elimination in Part 3 narrows what would fire this deferral to the *offline cross-check* role — certifying a rule's derivation once, on the bench, rather than certifying a candidate in the compiler — which is a smaller and more plausible milestone than the one the deferral was written against. The third clause remains unmet regardless: no pinned hermetic toolchain plan exists and no host installation has been authorized.

**Two facts for that spike's exit criteria, now read at the primary source rather than at the tool's README.** The spike's note that FPTaylor's checker "excludes FPTaylor's advanced power-of-two rounding model" is confirmed by the TOPLAS paper's own Section 6.2: "The general improved rounding model is not formally verified yet." And a fact that runs the other way, which the spike's framing does not carry: the same section reports that "The formalization of FPTaylor helped us to find a critical bug in our implementation" — a certificate mechanism is evidence about the tool, not only about the instance, which raises the value of the offline role this record leaves it.

## Open axes, each with a filed destination

Every axis below is a ticket or a deferral with a trigger. None is left as a note.

- **The distributivity premise ADR 0095 rested on is refuted by a caller.** → [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](../../../tickets/reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md). Tom's decision, not an agent's; the ticket carries the derivation and the price.
- **Rewriting through a transcendental's functional equation is an unnamed dimension.** → [`name-the-elementary-identity-rewrite-dimension`](../../../tickets/name-the-elementary-identity-rewrite-dimension.md), **answered** by [the elementary-identity rewrite dimension](elementary-identity-rewrite-dimension.md): the freedom is one dimension and it is a fourth, its refusal wording is specified, and no permission is proposed — its only caller is this record's own rewrite, which the distributivity decline independently blocks. That record also corrects one sentence of this one: the freedom is *elementary*-function rather than transcendental, because the square root's identity has the same shape and the square root is not transcendental.
- **The target accuracy authority cannot yield the numeric `eps_exp` a bound needs.** → [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](../../../tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md).
- **The tree-fold form of the bound is underived**, and flash-class kernels merge `(m, d)` pairs in a tree rather than a chain. The sequential derivation does not transfer by substituting the height, because a merge applies a different operation count. → [`derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound`](../../../tickets/derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound.md).
- **The bound is loose by one to two orders of magnitude**, and the sharpened `n*u` constants exist. → [`tighten-the-rescaling-bound-with-the-sharpened-summation-constants`](../../../tickets/tighten-the-rescaling-bound-with-the-sharpened-summation-constants.md), filed `deferred` with its trigger, because tightening a bound nothing consumes yet is work whose value depends on a caller being refused by the loose one.
- **Higham's monograph is unread.** → the `higham-asna-2002` row in [the source record](sources/README.md#higham-asna-2002), an acquisition request for Tom with what it would and would not decide.
- **Whether an analyzer ever becomes fast and covering enough to be an online tightener.** → recorded as the refutation condition in Part 3 and carried by the FPTaylor deferral's trigger log rather than by a new ticket, because a second ticket asking the same question is a second thing to sweep.

## What this record does not establish

- **No contract changed.** No permission was admitted, no tolerance vocabulary was registered, no ADR was accepted, and no crate changed. `implementation_status` is `not-started` and that is the honest value.
- **The derivation is unchecked by machine and rests on a survey rather than on the monograph that proves its foundation.** Both are stated where they matter and neither is hidden behind the `sound-proof` label.
- **The measurement is a bounded observation of a 22-case corpus in a Python simulation of binary32**, on one host, with a correctly rounded `exp` that no real target provides. It refutes; it does not confirm.
- **Nothing here establishes that the online-softmax rewrite is profitable.** It establishes what the rewrite costs numerically. Whether a one-pass fold is faster on any target is a cost-model and scheduling question this record does not touch, and the price being affordable is not an argument that the rewrite is wanted.
