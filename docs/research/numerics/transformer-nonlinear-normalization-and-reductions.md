---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"
kind: "research"
title: "Transformer non-linear, normalization, and reduction contracts"
topics: ["numerics", "softmax", "normalization", "activations", "masking", "reductions", "transcendentals", "transformer", "language-model", "metal"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.correctness-and-testing"]
depends_on: ["tiler.research.shapes.transformer-operation-and-shape-surface", "tiler.research.program-planning.first-metal-lm-workload"]
ticket: "scope-transformer-nonlinear-normalization-and-reductions"
---

# Transformer non-linear, normalization, and reduction contracts

**Status:** durable derivation record for rung L3′ of the language-model inference ladder. It is a research outcome, not a capability: nothing here registers an operation, admits an accuracy contract, or authorizes implementation. Every family it names sits at R2 on the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix), and this record moves no row.

## Traceability

- **Work record:** [`scope-transformer-nonlinear-normalization-and-reductions`](../../../tickets/scope-transformer-nonlinear-normalization-and-reductions.md).
- **Ladder position:** rung L3′ of [the roadmap's language-model ladder](../../roadmap.md#the-ladder). Its input is rung L2's [operation and shape surface derivation](../shapes/transformer-operation-and-shape-surface.md), which named softmax, RMS normalization, SiLU, and masking and explicitly handed them here rather than filing them. Its consumer is L4, [`design-attention-program-vertical`](../../../tickets/design-attention-program-vertical.md), which cannot assemble an attention block until each of these families has a formula.
- **Workload:** the L1 [workload profile](../program-planning/first-metal-lm-workload.md) — `Qwen/Qwen3-0.6B-Base` at revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`, widened to F32, batch 1, greedy, bounded by the C1 conformance row and the B1 benchmark matrix.
- **Governing contracts read as evidence, not edited beyond the hooks noted at the end:** [Numerical semantics](../../numerical-semantics.md) for the three-part contract, the transcendental accuracy vocabulary, the reduction contract, the extrema families, and the honesty rule; [Correctness and testing](../../correctness-and-testing.md) for conformance levels and the adversarial corpus; [Reduction semantics and legality](reduction-semantics-and-legality.md) for contributor order, seeds, empty domains, and the reassociation/permutation split; [Transcendental accuracy precedents](transcendental-accuracy-precedents.md) and [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) for what an accuracy contract may say; [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) for what a target may not be asked to substitute; [ADR 0087](../../decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) for the contraction identity this record deliberately does not reopen.
- **Inspected primary source:** `src/transformers/models/qwen3/modeling_qwen3.py` at `transformers` v4.51.0, verified by `shasum -a 256` against the digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2` that the workload profile's manifest records. Line numbers below are that file's.
- **Retained experiments:** [Metal transcendental emission probe](../../../spikes/numerics/metal_transcendental_emission/README.md) and [Transformer reference-semantics probe](../../../spikes/numerics/transformer_reference_semantics/README.md).
- **Evidence recorded after this record, which changes what D-4 still needs:** [Metal elementary-function accuracy guarantee](metal-elementary-function-accuracy.md), the normative half of the question the emission probe deliberately stopped short of. It also states which of this record's expectations Metal's flush-to-zero section reaches — softmax's underflow band and RMS normalization's subnormal row, not SiLU's `-88.73` band, which is an exact finite-over-infinity division rather than a flush.

Claims are labelled **Fact** when traced to inspected source at a verified digest or to a merged record, **Inference** when derived from stated facts, **Proposal** when not yet accepted or tested, and **Measurement** when tied to an exact environment and procedure.

## What L2 handed over, and what this rung adds

**Fact.** L2 delivered the family list and one disposition each: softmax and RMS normalization are atomic identities with a declared decomposition capability, SiLU is atomic, and causal masking is a composition over an additive mask input at short `T` with an absent predicate family at long `T`. L2 also derived that if softmax and RMS normalization pin their own formulas, this workload needs no standalone `Exp`, `Rsqrt`, `Maximum`-reduction, or `Divide` key.

**Inference — a disposition is not a contract, and the gap is the whole of this rung.** "Softmax is atomic" says one node carries the meaning; it does not say what the meaning *is*. Between the two sits every question that decides whether a kernel author writes the operation the model uses or a plausible neighbour: whether the row maximum is subtracted, whether the normalization divides or multiplies by a reciprocal, what the mask's fill value is, where `eps` sits, which of two SiLU spellings is meant, and what each does at a zero vector, an overflowing square, a fully masked row, and an empty domain. This record answers those, from inspected source and bounded measurement rather than from the shape of the formula as usually written.

**Inference — the reason to measure rather than to recall.** Every family here has at least two spellings a competent reader would call the same operation, and in F32 they are different operations. Two of the disagreements below were found only because a boundary input was added to a corpus that had reported uniform agreement without one: the two SiLU spellings differ at `-88.0` and nowhere else in the corpus, and the divide-versus-reciprocal question is invisible on most inputs. A uniform pass over a corpus that does not discriminate is the signature this repository distrusts, and it is the signature both of these questions produce by default.

## The required families, and their volume

**Inference — derived by multiplying the pinned per-layer structure by `num_hidden_layers: 28`.** `T` is the number of new positions in a pass and `S` the total context length.

| Family | Occurrences per forward pass | Scalar evaluations per forward pass | Reduced extent |
| --- | --- | --- | --- |
| Softmax over the key axis | 28 | 448·`T`·`S` exponentials | `S`, bounded symbolic and growing |
| RMS normalization | 113 — 57 over 1024, 56 over 128 | 144,384·`T` squares; 729·`T` reciprocal square roots | 1024 or 128, both static |
| SiLU | 28 | 86,016·`T` | none |
| Causal-mask application | 28 | 448·`T`·`S` additions | none |
| Attention scale | 28 | 448·`T`·`S` multiplications | none |

**Inference — the exponential is not a rare operation and its cost is not why it matters.** At the B1-d benchmark row's prefill (`T` = `S` = 8192) one forward pass evaluates 3.0 × 10¹⁰ exponentials. That is a performance fact. The correctness fact is the reduction count sitting under it: the 28 softmax occurrences cover 448·`T` rows in total and carry **two** reductions each, and the 113 normalization occurrences carry one each, so one forward pass contains 169 reductions that the single registered strict serial sum does not cover. Their accumulation order and dtype are unstated today, and [Reduction semantics and legality](reduction-semantics-and-legality.md) establishes that an unstated order is not a free choice but an unadmitted one.

**Fact — the 113 normalizations are two extent classes of one operation, not two operations.** `Qwen3Attention.__init__` at line 195 constructs `q_norm` and `k_norm` as `Qwen3RMSNorm(self.head_dim, eps=config.rms_norm_eps)` — the same class the decoder layer uses for `input_layernorm`, differing only in the normalized extent (128 rather than 1024) and therefore in the row count. The reference's own comment on that line reads "unlike olmo, only on the head dim!", which records that the *axis* is the distinguishing choice rather than the operation.

## Family contracts

Each subsection states the exact formula from inspected source, the dtype signature under the F32-widened workload, the conversion behaviour, the exceptional-value behaviour, and either an accuracy-or-order contract or a named unresolved decision. Where a subsection proposes rather than reports, it says so.

### Softmax

**Fact — the exact expression, from `eager_attention_forward` at lines 157–162.**

```python
attn_weights = torch.matmul(query, key_states.transpose(2, 3)) * scaling   # 157
if attention_mask is not None:
    causal_mask = attention_mask[:, :, :, : key_states.shape[-2]]          # 159
    attn_weights = attn_weights + causal_mask                              # 160
attn_weights = nn.functional.softmax(attn_weights, dim=-1, dtype=torch.float32).to(query.dtype)  # 162
```

**Fact — the scale is applied after the contraction, not folded into an operand.** Line 157 multiplies the contraction *result* by `scaling`, which `Qwen3Attention.__init__` sets at line 179 to `self.head_dim ** -0.5`. Pre-scaling `query` instead is a different F32 computation — it rounds `q · scale` once per query element and then rounds each product again, rather than rounding the accumulated score once. **Measurement.** `scaling` is the Python float `0.08838834764831845`, whose F32 rounding is `0x3db504f3`; it is not exactly representable, so the constant's own rounding is part of the contract rather than an implementation detail.

**Measurement — the reference subtracts the row maximum.** On the row `[1000.0, 1000.0]` the reference returns `[0x3f000000, 0x3f000000]` (a half each) while the naive quotient-of-exponentials form returns `[0x7fc00000, 0x7fc00000]` (NaN), because `exp(1000)` overflows F32. The two forms are therefore not interchangeable, and the difference is not a tolerance — it is finite against NaN.

**Measurement — the reference multiplies by the denominator's reciprocal; it does not divide.** Counting only elements at which `numerator / denominator` and `numerator * (1 / denominator)` produce different bits, over 20,000 random rows per width: at width 2, all 3,037 discriminating elements match the reciprocal form and none matches the division form; at width 3, all 7,224 do. From width 4 upward a third bucket appears — 2,349 of 12,123 at width 4 — matching neither form, which is the denominator's own accumulation order disagreeing with the naive sum and is why the narrow widths are the ones that isolate the question.

**Inference — that measurement is why the formula cannot be left to a composer.** [Numerical semantics](../../numerical-semantics.md) makes `reciprocal_math` a permission a program grants, precisely because replacing a division with a reciprocal multiplication changes results. The reference has already made that choice inside what it calls one operation. A Tiler `Softmax` that pins "divide" would diverge from the reference on ordinary inputs while looking correct; one that pins "multiply by the reciprocal" reproduces it and consumes no permission, because the multiplication is then the semantics rather than a relaxation of a division. The choice must live in the pinned formula. **Proposal — pin the reciprocal form**, and record the divergence from the division form as the reason.

**Proposal — the exact formula.** For each output coordinate, over the canonical contributor sequence of the single reduced axis:

```text
m  = MaximumNumber-reduction over the reduced axis          # extrema family fixed below
e_i = Exp(s_i - m)                                          # one Subtract and one Exp per contributor
d  = Sum-reduction of e_i over the reduced axis
r_i = e_i * (1 / d)                                         # one Divide of 1 by d, then one Multiply each
```

**Proposal — the dtype signature.** Operand, accumulator, and result are all `tiler::f32@1` under this workload. The two reductions carry separate accumulator declarations: the maximum's accumulator is F32 with no widening question, while the sum's accumulator is the one place where widening is a live choice, addressed under *Reductions* below. Line 162 passes `dtype=torch.float32` explicitly and then `.to(query.dtype)`; in an all-F32 realization both are identities, which is the sense in which the L1 profile's observation that F32 widening moves the program *toward* the reference applies here.

**Proposal — conversion behaviour.** None is observable inside the operation at F32. The `.to(query.dtype)` at line 162 is an identity here and is *not* an identity at BF16, so the record states it rather than deleting it: the conversion boundary exists in the reference's semantics and a future narrower profile inherits it.

**Exceptional values.** Each row below is a decision, not a description of what happens to fall out.

| Case | Reference behaviour | Proposal for the Tiler contract |
| --- | --- | --- |
| Exponent overflow | Unreachable. After subtracting the row maximum every argument is ≤ 0, and **Measurement** puts the F32 overflow threshold at `0x42b17218` (≈ 88.72), above every reachable argument. | Reachability is a consequence of the pinned formula, not a separate guarantee. A contract that omitted the maximum subtraction would have to state overflow behaviour; this one need not, and that is a reason to pin the subtraction rather than to leave it to a permission. |
| Exponent underflow | **Measurement.** `Exp` returns a subnormal below argument `0xc2aeac50` (≈ −87.34) and exactly `+0.0` below `0xc2cff1b5` (≈ −103.97). A contributor more than ~104 below the row maximum contributes exactly zero. | Stated, not prevented. On the qualified Metal row F32 subnormals flush, so the subnormal band collapses to zero there and the contract must say the flush is a declared realization rather than an error — [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) forbids substituting a different contract to avoid it. |
| Masked contributor | **Measurement.** With the reference's own finite mask fill, `softmax([0.5, min, min])` returns exactly `[0x3f800000, 0x00000000, 0x00000000]` — one and two positive zeros. | The mask's fill value is part of the mask contract, not the softmax contract. Softmax owes only that an argument this far below the maximum reaches exactly zero, which follows from the underflow row. |
| Fully masked row | **Measurement.** Under the reference's finite fill the row returns uniform `0x3eaaaaab` (a third each); under a `-inf` fill the same row returns NaN. The two mask conventions disagree observably and only here. | **Unresolved decision D-1**, named below. The workload does not reach the case, and a contract that silently inherits either answer is a contract nobody chose. |
| Empty reduced axis | **Measurement.** The result has the same shape as the input, so a zero-length key axis yields a zero-length output and no scalar softmax is evaluated. | Softmax is shape-preserving, not shape-reducing, so its empty case is vacuous where a `Sum`'s is not. The contract states this, because the reduction contract's empty-domain rules would otherwise appear to apply and would produce the wrong obligation. |
| NaN contributor | Not measured; the maximum family's NaN rule decides it and the two admitted families disagree. | Follows from the extrema-family choice, which is **unresolved decision D-2**. |

**Proposal — the order contract.** Both embedded reductions are strict ordered folds over the canonical contributor sequence unless a registered permission authorizes otherwise, which is the same position [ADR 0087](../../decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) item 5 takes for the contraction's embedded reduction. Reassociation and permutation stay independent, so a SIMD-group tree over the key axis needs reassociation and a lane-strided partition needs permutation as well — a distinction [Reduction semantics and legality](reduction-semantics-and-legality.md) already fixes and this record does not restate.

**Inference — the sum reduction is the one whose order is observable, and the maximum's is not.** Floating-point maximum is associative and commutative for every input on which its family is total, so any tree over the same contributors gives the same bits; the sum is neither. That asymmetry is a legality fact worth stating explicitly, because it means a fused softmax may parallelize its first pass freely and its second pass only under a permission — and a schedule that assumed one permission covered both passes would be wrong in exactly one direction.

### RMS normalization

**Fact — the exact expression, from `Qwen3RMSNorm.forward` at lines 71–76.**

```python
input_dtype = hidden_states.dtype                                              # 71
hidden_states = hidden_states.to(torch.float32)                                # 73
variance = hidden_states.pow(2).mean(-1, keepdim=True)                         # 74
hidden_states = hidden_states * torch.rsqrt(variance + self.variance_epsilon)  # 75
return self.weight * hidden_states.to(input_dtype)                             # 76
```

**Fact — three separable decisions the usual spelling hides.** First, `variance` is a mean of *squares*, not a variance: nothing is subtracted, which is what makes this RMS normalization and not layer normalization. Second, `eps` is added to that mean **inside** the `rsqrt` argument, at line 75, rather than to the root outside it. Third, the operation uses `rsqrt` rather than `1 / sqrt`; these are different F32 results and different Metal intrinsics.

**Fact — the weight multiply happens after the cast back, not before.** Line 76 is `self.weight * hidden_states.to(input_dtype)`, so the narrowing conversion precedes the scaling. At F32 the conversion is an identity and the ordering is unobservable; at BF16 it is a rounding boundary between the normalization and the weight. The record states the order because a future narrower profile inherits it and because the F32 realization is the one that makes the question invisible.

**Proposal — the exact formula**, for each output row over the single normalized axis of static extent `N` (1024 or 128):

```text
q_i = x_i * x_i                       # N products, one rounding each
a   = Sum-reduction of q_i            # N contributors
u   = a * (1 / N)                     # N is a static power of two here, so this is exact
t   = u + eps                         # eps = 1e-6, added inside the rsqrt argument
r   = Rsqrt(t)
y_i = w_i * (x_i * r)                 # normalize, then weight; the weight is a Broadcast operand
```

**Inference — the division by `N` is exact for this workload and must not be assumed exact in general.** Both extents, 1024 and 128, are powers of two, so multiplying by `1/N` is an exact scaling of the exponent and introduces no rounding. A contract that pinned "multiply by the reciprocal extent" would silently acquire a rounding at a non-power-of-two extent, so the formula states a division whose exactness at these extents is a derived property rather than the definition.

**Proposal — the dtype signature.** Operand, weight, accumulator, and result are `tiler::f32@1`. The sum-of-squares accumulator is the same open widening question as softmax's denominator, and with a stronger case: 1024 contributors of magnitude-squared terms is a longer accumulation than the workload's other reductions and the one most exposed to it.

**Exceptional values.**

| Case | Behaviour | Consequence |
| --- | --- | --- |
| Zero row | **Measurement.** `rsqrt(0 + 1e-6)` is `0x4479ffff` (≈ 999.99994, not 1000), and the row normalizes to zeros. Total, no NaN. | The `eps` placement is what makes this total. **Measurement — the perturbation.** Removing `eps` from the `rsqrt` argument turns the zero row into four NaNs and a subnormal row into positive infinities. `eps` is a semantic term, not a numerical fudge, and a contract that treated it as optional would change the operation's domain. |
| Subnormal row | **Measurement.** A row of `1e-40` normalizes to `0x02081cb9` (≈ 1.0 × 10⁻³⁷), a *normal* result from subnormal inputs. | On the qualified Metal row, input subnormals flush to zero, so the same row normalizes to zeros there. This is a named divergence between the CPU reference and the declared target realization, at a case the reference reaches; it is not a defect to be tuned away and it is exactly the "subnormal and near-zero-variance vectors" obligation L2 assigned to this family. |
| Large row | **Measurement.** Squaring overflows at `0x5f7fffff` (≈ 1.845 × 10¹⁹). A row of `1e20` gives a mean of squares of `0x7f800000` (positive infinity), `rsqrt(inf)` of zero, and a result of **all positive zeros**. | This is the silent-wrongness case of the family. The output is finite, plausible, and wrong: a normalization that should return a unit-RMS row returns zeros, with no NaN and no infinity to reveal it. The contract states the threshold, and the conformance corpus must contain a row above it. Whether the operation refuses is **unresolved decision D-3**. |
| Mixed infinite or NaN element | Not measured. `inf * 0` from the overflow path is NaN, so a row containing one infinity and one finite element propagates differently from an all-large row. | Follows from the scalar `Multiply` contract once the accumulator's own infinity behaviour is fixed; no separate rule. |
| Empty normalized axis | Unreachable: both extents are static and nonzero. | The family still owes a declared behaviour, because the extent is an attribute and not a proof. A zero extent makes the mean-of-squares an empty sum, whose `+0.0` empty result would give `rsqrt(eps)` and a shape-empty output — vacuous, like softmax's. Stated so that it is decided rather than discovered. |

**Inference — the eps constant is part of identity, not configuration.** `rms_norm_eps` is `1e-06` at the pinned revision and it is *not* exactly representable in F32, so the operation's canonical attributes must carry its exact bits. Two RMS normalizations differing only in that constant are different operations and must not share an identity, a cache subject, or a golden.

### SiLU

**Fact — the workload's activation is SiLU and nothing else.** `Qwen3MLP.__init__` at line 91 sets `self.act_fn = ACT2FN[config.hidden_act]`, `config.json` declares `hidden_act: "silu"`, and **Measurement** resolves that name through the reference's own table to `torch.nn.modules.activation.SiLU`. The MLP at line 94 is `down_proj(act_fn(gate_proj(x)) * up_proj(x))`.

**Inference — GELU does not appear in this workload, in either form, and saying so is the finding.** The ticket asks for exact-versus-tanh GELU to be separated because the two are different semantic operations that share a name. They are; and the separation this workload needs is not between them but *away* from them. A kernel author who imports a GELU contract here implements a different activation, and the erf-versus-tanh question — genuinely live for a GPT-2-shaped fixture, which the L1 profile keeps as a diagnostic family — is not this workload's question and must not be answered on its behalf. [`docs/ir.md`](../../ir.md) already fixes the general rule that an admitted `Gelu` key pins its exact formula or decomposition; the reason it is quoted here is to record that this record does not admit one.

**Measurement — the two ordinary spellings of SiLU are not the same F32 operation.** Over the corpus `[-100, -88, -20, -1, -0.0, +0.0, 1, 20, 100, -inf, +inf, NaN]`, `x / (1 + exp(-x))` agrees with the reference at every input, while `x * sigmoid(x)` differs at exactly one: `-88.0`, where it returns `0x83354ddb` against the reference's `0x83354ddc`, one ULP apart. An earlier corpus without an input near the exponential's overflow threshold reported all three spellings identical.

**Proposal — pin the division form**, `y = x / (1 + Exp(-x))`, because it is the one that reproduces the reference bit-for-bit on the measured corpus. **Inference — and state the boundary rather than the conclusion.** Twelve inputs are not a proof that the two forms agree everywhere else, and one counterexample is a proof that they are different operations. The claim this record makes is the second, which is the one the contract needs; the first is not made.

**Proposal — the dtype signature.** Operand and result `tiler::f32@1`, one rounding at the negation, one at the exponential per its accuracy contract, one at the addition, one at the division.

**Exceptional values.** **Measurement**, all from the reference and reproduced by the division spelling: `silu(-0.0)` is `-0.0` and `silu(+0.0)` is `+0.0`, so signed zero is preserved and the operation is not zero-canonicalizing. `silu(x)` is exactly `-0.0` for `x` at or below about `-88.73`, because `exp(-x)` overflows to infinity there and the quotient underflows — the mathematically correct value in that band is a subnormal, and the qualified Metal row flushes subnormals to zero anyway, so the two routes agree on this target and would not agree on a preserving one. `silu(+inf)` is `+inf`. **`silu(-inf)` is NaN**, in all three spellings including `torch.nn.SiLU`: the quotient is `-inf / +inf`. The operation is therefore *not* total on the extended reals, which is a fact about the reference rather than an accident of the spelling, and `-inf` does not occur in this workload's MLP.

**Proposal — the accuracy contract is the exponential's, plus three exact roundings.** Under ADRs 0016 and 0042 the family owes a typed accuracy contract over an immutable reference. The only inexact element is `Exp`; the negation, the addition of one, and the division each have exact round-to-nearest-ties-to-even contracts already fixed by [ADR 0024](../../decisions/0024-initial-arithmetic-rounding.md). So SiLU's contract is a composition whose only open tolerance is the exponential's — which is **unresolved decision D-4**, shared with softmax.

### Causal-mask application

**Fact — the mask is an additive F32 tensor whose fill value is finite.** `_prepare_4d_causal_attention_mask_with_cache_position` builds it at lines 730–742: `torch.full((sequence_length, target_length), fill_value=min_dtype, …)` with `min_dtype = torch.finfo(dtype).min`, then multiplies elementwise by the boolean `diagonal_attend_mask` at line 742, where `diagonal_attend_mask[i, j]` is `j > cache_position[i]`.

**Measurement — the two entry values, exactly.** A masked entry is `0xff7fffff`, the most negative finite F32 (`−3.4028235 × 10³⁸`). An attended entry is `min_dtype * False`, which is `0x80000000` — **negative** zero, not positive zero and not a written zero.

**Inference — this corrects the reading a competent author would otherwise bring.** "Additive causal mask" is almost universally implemented and described with `-inf`, and the reference does not use it. The difference is not cosmetic: it decides the fully-masked-row behaviour (uniform against NaN, measured above), it keeps every masked score finite so that `score + mask` never forms `inf - inf`, and it means the mask's own value participates in the score's rounding rather than saturating it. A Tiler mask contract that wrote `-inf` would produce NaN rows where the reference produces uniform ones, and would do so only on inputs the C1 row does not contain.

**Inference — the attended entry's sign is observable exactly once.** Adding `-0.0` to any score `s` returns `s` for every `s` except `s = +0.0`, where `+0.0 + (-0.0)` is `+0.0`, and `s = -0.0`, where the result stays `-0.0`. So the addition is value-preserving on this workload's data. It is still a semantic operation with a rounding boundary, and a rewrite that deleted it as an identity would be consuming a signed-zero relaxation it was never granted.

**Fact — no row of this workload is fully masked.** With causal masking, batch 1, and no padding token, row `i` attends to positions `0..i`, so every row has at least one attended entry. The reference's repair path for fully masked rows, `AttentionMaskConverter._unmask_unattended`, is guarded at lines 676–685 to the `sdpa` implementation on `cuda` or `xpu` and does not run for the eager CPU path the C1 fixture used. So the case is unreachable here *and* unrepaired in the reference, which is why decision D-1 is a decision rather than an inherited answer.

**Proposal — the disposition.** L2's crossover stands and this record adds the value evidence to it rather than reopening it: supplying the mask as an F32 program input needs only `Broadcast` and `Add`, both already required, and costs `T × S × 4` bytes — 720 at C1, 268,435,456 at B1-d. Deriving it from iteration coordinates needs an index-domain comparison and a selection over F32 values, and [ADR 0084](../../decisions/0084-reference-canonical-index-expressions-from-domain-predicates.md)'s predicate vocabulary excludes the first by construction while the registry admits no boolean dtype for the second. The input route is correct at every declared row; the derived route is a tracked capability whose trigger is a row where the mask outgrows the program.

### Reductions

**Inference — the reductions inside softmax and normalization are the same two shapes, and only one of them is order-sensitive.**

| Reduction | Where | Contributors | Combiner | Order sensitivity |
| --- | --- | --- | --- | --- |
| Row maximum | softmax, 28 occurrences covering 448·`T` rows in total | `S`, bounded symbolic and growing | an extrema family, **D-2** | none: associative and commutative wherever its family is total |
| Row sum of exponentials | softmax, the same 448·`T` rows | `S` | `Add` under [ADR 0024](../../decisions/0024-initial-arithmetic-rounding.md) | full: neither associative nor commutative in F32 |
| Row sum of squares | RMS normalization, 113 occurrences covering 729·`T` rows in total | 1024 or 128, static | `Add`, with an elementwise squaring prologue | full |

**Fact — none of these is the registered reduction.** The only registered reduction key is `tiler::strict-serial-sum-f32@1`, holding the sole `OrderedReduction` fusion role, so a maximum reduction and a prologue-carrying sum resolve to no fusion legality at all. The support matrix's own row for reductions beyond strict sum records this and places it at R2.

**Inference — the sum-of-squares is a fused prologue, not a new reduction family.** `mean(x²)` is `Sum` over a squaring prologue, which is the shape `OrderedReduction` was defined for and which [Correctness and testing](../../correctness-and-testing.md)'s reduction matrix already lists as required coverage ("fused prologue and epilogue expressions"). What it needs is a role for the family it belongs to, not a new family.

**Proposal — the accumulation dtype is an explicit part of each reduction's signature and this workload does not settle it.** [Numerical semantics](../../numerical-semantics.md) requires the accumulator dtype to be declared and states that it does not by itself determine reduction semantics. Three separate observations bear on the choice and none of them decides it: the reference accumulates in F32, so an F32 accumulator is what reproduces it; the L1 profile's measured reference-sensitivity envelope attributes the dominant divergence on the C1 row to *contraction* reduction order rather than to these reductions, so widening here would buy little on that row; and the sum-of-squares over 1024 magnitude-squared terms is the longest accumulation in the family set and is the one where the argument would be strongest at a longer context. This is **unresolved decision D-5**, and it belongs to [`implement-parallel-reduction-strategies`](../../../tickets/implement-parallel-reduction-strategies.md), which already owns making the accumulation dtype explicit and rejecting a narrower one with a typed reason.

## Small tensor examples

**Measurement — every bit pattern below was computed in the pinned environment, not derived by hand.**

### Softmax over a four-position key axis with one masked position

Logical operation: `Softmax` over the last axis of a `[1, 4]` score row, after the scale and the mask add. Scores `[1.0, 2.0, 3.0, min]`, where `min` is the mask fill.

| Step | Values (F32 bits) |
| --- | --- |
| scores `s` | `0x3f800000` `0x40000000` `0x40400000` `0xff7fffff` |
| row maximum `m` | `0x40400000` (3.0) |
| shifted `s − m` | `0xc0000000` `0xbf800000` `0x00000000` `0xff7fffff` |
| `e = Exp(s − m)` | `0x3e0a9555` `0x3ebc5ab2` `0x3f800000` `0x00000000` |
| denominator `d = Sum(e)` | `0x3fc06957` (≈ 1.5032147) |
| result `e · (1/d)` | `0x3db861f2` `0x3e7a9a18` `0x3f2a4d3a` `0x00000000` |

Logical properties: shape-preserving, one reduced axis of extent 4, two embedded reductions. The masked position contributes exactly `+0.0` to the denominator and receives exactly `+0.0`, which is the property that makes a finite mask fill behave like an exclusion without being one. **Measurement — the outputs sum to `0x3f7ffffe`, not to `0x3f800000`.** Softmax does not produce a row summing to exactly one in F32, and a conformance check that asserted it would fail on the reference.

Candidate physical plans, none selected here: one fused kernel carrying max, exponential, sum, and scale in a single pass over the row; three dispatches materializing `m`, then `e`, then `d`; or a two-pass online form that fuses the maximum and the sum. The third is a legal alternative only under reassociation, because rescaling a running sum when the maximum changes regroups the contributor sequence — which is a legality question, not a cost one, and is the reason it is named here rather than assumed.

### RMS normalization over a two-wide axis

Logical operation: `RmsNorm` over the last axis of a `[1, 2]` row, `x = [3.0, 4.0]`, weight `w = [1.0, 2.0]`, `eps = 1e-6`.

| Step | Values (F32 bits) |
| --- | --- |
| squares `x²` | `0x41100000` (9.0) `0x41800000` (16.0) |
| mean of squares | `0x41480000` (12.5) |
| `+ eps` | `0x41480001` (12.500001) |
| `Rsqrt` | `0x3e90d0c2` (≈ 0.2828427) |
| normalized `x · r` | `0x3f593923` `0x3f90d0c2` |
| weighted `w · (x · r)` | `0x3f593923` `0x4010d0c2` |

Logical properties: shape-preserving, one reduced axis of extent 2, one embedded reduction, one `Broadcast` operand (the weight, `[2]` against `[1, 2]`). **Measurement — `eps` changes the result at this ordinary input**, not only at the zero row: dropping it changes the normalized bits. So `eps` is not a guard that activates near zero; it perturbs every output, and a contract that described it as a guard would be describing a different operation.

Candidate physical plans: one fused kernel over the row; or a sum-of-squares dispatch followed by a scaling dispatch, which materializes the reciprocal square root per row and is the plan a long normalized axis would consider. Both preserve the same rounding boundaries; neither is selected here.

## Composite spelling, atomic operation, and fused implementation

**Inference — the three are different questions and this record answers each separately.** L2 assigned each family an identity disposition; the columns below add what that disposition costs and what it does not decide.

| Family | Graph spelling if composed | Atomic identity — justified? | Fused physical implementation |
| --- | --- | --- | --- |
| Softmax | `Maximum`-reduction → `Subtract` → `Exp` → `Sum`-reduction → `Divide`-of-one → `Multiply`: six nodes, five of them families with no key | **Yes.** The composition does not determine the answer — the maximum subtraction and the reciprocal form are both composer choices with observable F32 consequences, and both were established here by measurement rather than by reading the composition. One key pins both. | A single-pass or two-pass kernel over the reduced axis, legal only where the sum's order permission covers the chosen topology. The maximum pass may parallelize freely; the sum pass may not. |
| RMS normalization | `Multiply` → `Sum`-reduction → `Multiply`-by-reciprocal-extent → `Add` → `Rsqrt` → `Multiply` → `Broadcast` → `Multiply` | **Yes**, same derivation: `eps` placement, `rsqrt` against `1/sqrt`, and the exact `eps` bits are three separately observable decisions a composition leaves open. One key with a reduced-axis attribute covers all 113 occurrences. | One kernel per row; the weight enters as a broadcast operand rather than a materialized tensor. |
| SiLU | `Negate` → `Exp` → `Add`-one → `Divide` | **Yes**, but for a different reason: the composition *would* determine the answer, since all four steps have fixed contracts. The justification is that the two conventional spellings are one ULP apart at a measured input, so a graph that spells it as `x * sigmoid(x)` is a different operation from one that spells it as the quotient — and only a key can say which the model uses. | `ElementwiseArithmetic`, fusable into either adjacent contraction's epilogue or the SwiGLU multiply. |
| Causal masking | `Broadcast` → `Add` over an F32 mask input | **No.** Both families are already required by the workload, the composition is two nodes, and neither node has a composer choice: the mask's values come from outside the program. | Fuses into the score kernel's epilogue; at long `T` the alternative is a derived predicate, which is absent. |
| Attention scale | `Constant` → `Multiply` | **No.** Already at R6 per the support matrix; the only decision is that the scale applies to the score rather than to an operand, which is a graph-position fact and not an identity. | Folds into the contraction epilogue. |

**Inference — what the three atomic identities remove, restated with the cost.** L2 derived that atomic `Softmax` and `RmsNorm` remove any need for standalone `Exp`, `Rsqrt`, `Maximum`-reduction, and `Divide` keys. That remains true and this record adds the invoice: each atomic key owes a complete formula, a subordinate transcendental contract, an accumulator declaration, an order contract, and an exceptional-value policy — five obligations that the composition would also have owed, merely spread across four families instead of concentrated in one. The saving is in the number of accuracy contracts to admit, not in the amount of thinking.

## Metal feasibility of the required transcendentals

**Measurement — the boundary first, because it is narrow.** The probe compiles; it does not execute. No device was opened, no kernel ran, and no result bit pattern was compared. What follows establishes which AIR intrinsic the offline compiler selects for a spelling under a flag set, at offline compiler `metalfe-32023.883`, `-std=metal4.0`, `-target air64-apple-macos26.0`, on macOS 27.0 build `26A5388g`. Every accuracy, exceptional-value, and delivered-numerics question about these intrinsics remains `Unknown` after it. The retained record is [`spikes/numerics/metal_transcendental_emission/results/2026-07-31-air-emission-msl4-macos26-metal32023.883/`](../../../spikes/numerics/metal_transcendental_emission/results/2026-07-31-air-emission-msl4-macos26-metal32023.883).

**Fact — the primitives exist as named intrinsics, and the gap L2 recorded is a Tiler-side gap.** L2 recorded that "no exponential exists in the structured-kernel vocabulary" and "no reciprocal square root exists" — statements about `crates/tiler-ir/src/kernel/model.rs`, not about Metal. Metal supplies `air.exp.f32`, `air.rsqrt.f32`, `air.sqrt.f32`, `air.fmax.f32`, `air.simd_sum.f32`, and `air.simd_max.f32`. So the families are emittable; what is missing is on Tiler's side of the boundary.

**Fact — there is no sigmoid and no SiLU in Metal.** The exact check: `grep -rl sigmoid` over the toolchain's `include/metal` directory returns nothing, and a kernel calling `sigmoid(x)` fails to compile with `use of undeclared identifier 'sigmoid'`. Both SiLU spellings are compositions in MSL exactly as they are in the semantic graph, which is a convenience: the lowering of an atomic `SiLU` key has no native intrinsic to be tempted by.

**Measurement — under the governed baseline every spelling selects its precise intrinsic and no call carries fast-math flags.**

| MSL spelling | AIR callee under `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off` | Call-site fast-math flags |
| --- | --- | --- |
| `exp(x)` | `air.exp.f32` | none |
| `precise::exp(x)` | `air.exp.f32` | none |
| `fast::exp(x)` | `air.fast_exp.f32` | none |
| `rsqrt(x)` | `air.rsqrt.f32` | none |
| `precise::rsqrt(x)` | `air.rsqrt.f32` | none |
| `fmax(a, b)` | `air.fmax.f32` | none |
| `a / b` | LLVM `fdiv` | none |
| `simd_sum(x)` / `simd_max(x)` | `air.simd_sum.f32` / `air.simd_max.f32` | none |

**Measurement — the compiler default is not the governed baseline, and the difference is a different intrinsic.** With no flags at all, or with `-fmetal-math-fp32-functions=fast`, or with `-fmetal-math-mode=fast`, the unqualified `exp` selects `air.fast_exp.f32`, `rsqrt` selects `air.fast_rsqrt.f32`, `fmax` selects `air.fast_fmax.f32`, and every call carries LLVM `fast` flags. The mechanism is visible in the header: `metal_math` defines `__METAL_MAYBE_FAST_MATH__` as `__METAL_FAST_MATH__` when `__METAL_MATH_FP32_FUNCTIONS_FAST__` is defined and as `__METAL_PRECISE_MATH__` otherwise, and the compiler predefines that macro by default.

**Inference — this fills a gap the Apple numerical-behaviour record named rather than contradicting it.** That record swept `-fmetal-math-fp32-functions` over multiply, add, divide, and a fused multiply-add, found neither the emitted module nor any bit pattern moved, and stated the boundary precisely: the flag governs the F32 *function* implementations and that matrix called none, so "`sin`, `sqrt`, `rsqrt`, and every other function it actually governs remain unmeasured." The flag is inert for the operations it was swept over and live for the ones it was not. Both statements are now measured, and the governed baseline the workload profile records already selects the precise family — which is a confirmation, not a discovery, and it is worth having as evidence rather than as an assumption.

**Measurement — the two flags are different axes, which matters for what a Tiler profile may declare.** `-fmetal-math-mode=relaxed` selects the `fast_` intrinsics *without* attaching `fast` flags to the call sites, while `-fmetal-math-mode=fast` does both. Separately, `metal::precise::exp` emits `air.exp.f32` under every flag set tested — the namespace is a stronger authority than the flag — but under the fast modes the call still carries `fast` flags, which is a freedom LLVM may exploit around the call rather than inside the intrinsic. **Inference.** Intrinsic selection and call-site fast-math licence are two independent things a backend may grant, so a target profile that recorded one "fast math" bit would conflate them, which is the failure [Numerical semantics](../../numerical-semantics.md) already forbids when it says one permission never implies another.

**Inference — feasibility is established for emission and for nothing else.** A structured-kernel `Exp` or `Rsqrt` construct can be lowered to a named intrinsic under the governed flags. Whether that intrinsic satisfies any accuracy contract Tiler would state is unmeasured and, under [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md), cannot be established by a spot check: what would establish it is an applicable normative guarantee from Apple's own specification, exhaustive evaluation over a tractable domain, or a proof — and an empirical maximum error, however carefully sampled, would remain empirical. That is **unresolved decision D-4**, and it is the single largest gap between this record and an admissible `Softmax`, `RmsNorm`, or `SiLU` key. **The first of those three routes was subsequently taken**: [Metal elementary-function accuracy guarantee](metal-elementary-function-accuracy.md) quotes §8.4 of the retained specification, which was on disk when this paragraph was written and unread, and D-4's entry below records what the quoted numbers do and do not settle.

## What generic work already covers, and what it does not

**Inference — three of this record's requirements are already owned elsewhere and this record deliberately files nothing for them.**

- **Reduction order, accumulation dtype, and parallel topology.** [`implement-parallel-reduction-strategies`](../../../tickets/implement-parallel-reduction-strategies.md) owns making the accumulation dtype explicit, keeping reassociation and permutation independent, and rejecting a narrower accumulator with a typed reason. This record supplies the workload evidence — 169 reductions per forward pass across 141 operation occurrences, over two combiners, one order-insensitive and one not — and cross-links rather than restating the contract.
- **The typed accuracy-contract vocabulary.** [ADR 0042](../../decisions/0042-use-typed-transcendental-accuracy-contracts.md) already admits correctly-rounded, faithful, piecewise-bounded, and named-behaviour forms with exact rational tolerances and the `tiler::ulp-reference-gap@1` metric. Nothing here needs a new *kind* of contract; what is missing is a chosen tuple, which is [Q-SEM-004](../../open-questions.md#q-sem-004--first-profile-transcendental-tuples)'s and D-4's.
- **The permission algebra.** Reassociation, permutation, reciprocal math, approximate intrinsics, and subnormal handling are already independent dimensions with resolved per-operation semantics. This record consumes them; it proposes no new dimension.

**Inference — two requirements are not covered by anything.** The extrema *family* choice for the softmax maximum has an accepted ADR ([0023](../../decisions/0023-floating-point-extrema-semantics.md)) and no key, and its reduction form is a separate obligation from its elementwise form. And no work anywhere owns the mask's fill-value convention, because until this record the mask was described as additive without its value being read.

## Typed refusals

Each unsupported case below must reject with a typed, explainable diagnostic rather than approximate, because each is a place where a silent approximation returns a plausible tensor.

- A softmax whose reduced axis is a symbol with no proved upper bound refuses rather than compiling a generic program, on the same rule L2 recorded for `S`.
- A softmax or normalization whose reduction order permission does not cover the selected topology refuses, naming the missing dimension — reassociation or permutation — rather than reporting a generic illegality. The maximum pass and the sum pass are checked separately, because one may be legal while the other is not.
- An accumulator narrower than the declared contract refuses with a typed reason and never silently narrows a partial in scratch.
- A transcendental whose resolved accuracy contract no installed target realization refines refuses, carrying the declaring profile's identity and the measurement boundary of the refusing fact. Selecting `air.fast_exp.f32` to satisfy a contract stated against the precise family is exactly the substitution [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) forbids, and the emission measurement above shows it is one compiler flag away by default.
- A `Softmax` or `RmsNorm` occurrence whose reduced axis is absent, duplicated, or out of range refuses at construction, per rule, so the diagnostic names which rule.
- An `RmsNorm` with a non-positive, non-finite, or NaN `eps` refuses at construction. The zero case is the one that matters: it is not a degenerate parameter but a different operation, whose domain excludes the zero row.
- A masked softmax whose mask fill value disagrees with the declared convention is not detectable and therefore is not a refusal — it is a conformance obligation. Stated here so that the absence of a check is deliberate rather than an oversight.

## Unresolved decisions

Each carries what would close it. None is a routine implementation detail, and none is being escalated ahead of the evidence that would settle it.

- **D-1 — the fully masked row.** Uniform distribution (the finite-fill reference behaviour), NaN (the `-inf` behaviour), or a typed refusal. **Closes when** a workload reaches the case; this one does not, because causal masking with batch 1 and no padding leaves every row at least one attended position. Recording it as unreached is the point: a contract that inherited either answer would have inherited it from whichever mask convention the implementer happened to write.
- **D-2 — the extrema family for the softmax row maximum.** `Maximum` propagates NaN; `MaximumNumber` prefers the numeric operand. [ADR 0023](../../decisions/0023-floating-point-extrema-semantics.md) makes them separate operations and [Numerical semantics](../../numerical-semantics.md) records that Metal `fmax` is number-preferring with an order-dependent signed-zero result, so neither lowers to `air.fmax.f32` without a fixup or a matching authorized relaxation. **Closes when** the softmax key is admitted; the choice determines whether one NaN score poisons its whole row.
- **D-3 — whether RMS normalization refuses on a squaring overflow.** The reference returns all zeros silently at any row whose elements reach `0x5f7fffff` in magnitude. A refusal is safer and is a semantic precondition requiring a proof or a runtime scan, which [Numerical semantics](../../numerical-semantics.md) records is itself a costed operation rather than a free guard. **Closes when** the normalization key is admitted, with the conformance corpus carrying a row above the threshold either way.
- **D-4 — the accuracy contract for `Exp` and `Rsqrt`, and therefore for all three atomic families.** This is [Q-SEM-004](../../open-questions.md#q-sem-004--first-profile-transcendental-tuples) instantiated on a named workload. **Closes when** an applicable normative guarantee, an exhaustive evaluation over a tractable domain, or a proof establishes what the selected Metal intrinsics deliver; the emission probe establishes which intrinsic is selected and deliberately stops there. **Partially supplied, and the shape of the remainder changed.** [Metal elementary-function accuracy guarantee](metal-elementary-function-accuracy.md) reads the guarantee out of the retained specification rather than a device: Table 8.1 gives `exp` `<= 4 ulp`, `rsqrt` correctly rounded, and `x + y`, `x - y`, `x * y`, `1.0 / x`, `x / y`, and `fma` correctly rounded at F32, identically in both retained revisions, and the applicability of that table under the governed flags rests on the specification's own statement that `-fno-fast-math` is equivalent to `-fmetal-math-fp32-functions=precise` and `-fmetal-math-mode=safe`. Two derivations remain before any of it may be written into a contract, and they bind disjoint halves of the set: Apple states its ULP bound under its own definition of `ulp`, whose representable case is ambiguous where `tiler::ulp-reference-gap@1` resolves it, so `Exp` needs a registered cross-metric implication with a factor of 2, 3, or 1 depending on the reading and the domain; and §8.2 permits either round-to-nearest-ties-to-even or round-toward-zero, so every correctly rounded entry — `Rsqrt` and the whole arithmetic set — cannot be spelled as the carrier's only `ReferenceRoundingRule`. The exceptional-value, signed-zero, and subnormal policies this record's tables leave open stay `Unknown` from the specification either way.
- **D-5 — the accumulation dtype for the two sum reductions.** F32 reproduces the reference; a wider accumulator would not. **Closes** inside [`implement-parallel-reduction-strategies`](../../../tickets/implement-parallel-reduction-strategies.md), which already owns making the choice explicit; this record supplies the extents and the contributor counts that the choice needs.

## Capability tickets filed from this derivation

Three verticals, dependency-ordered, each carrying reference behaviour, compiler legality, Metal realization, explainable refusal, and bounded conformance evidence. They are verticals rather than one ticket per crate and rather than one ticket per module, and the ordering is a dependency claim rather than a priority one.

| Order | Ticket | Outcome |
| --- | --- | --- |
| 1 | [`admit-the-silu-activation-family`](../../../tickets/admit-the-silu-activation-family.md) | The first transcendental family in the project reaches an executable target — one operand, one result, no reduction, one exponential — so that the accuracy-contract machinery ADRs 0016 and 0042 accepted is exercised end to end before a reduction-carrying family depends on it. |
| 2 | [`admit-the-rms-normalization-family`](../../../tickets/admit-the-rms-normalization-family.md) | The 113 normalizations per forward pass become expressible: a reduced-axis attribute, an exact `eps` in identity, a reciprocal square root, and a broadcast weight, over the static extents 1024 and 128. |
| 3 | [`admit-the-softmax-family`](../../../tickets/admit-the-softmax-family.md) | The 28 attention softmaxes become expressible: two reductions with different order sensitivity, over the workload's one growing symbolic extent. |

**Inference — why SiLU is first and softmax last.** SiLU has one transcendental and no reduction; RMS normalization adds one order-sensitive reduction over a static extent; softmax adds a second reduction, a second combiner family, and the only symbolic growing extent in the set. Each rung adds exactly one new obligation to the one below it, so a failure attributes to the obligation that was added. Ordering them the other way would make the first delivery the one with the most simultaneous unknowns.

**Inference — masking files nothing, and that is a result.** The input route composes `Broadcast` and `Add`, both of which [`admit-the-reindex-and-broadcast-operation-families`](../../../tickets/admit-the-reindex-and-broadcast-operation-families.md) already delivers and `tiler::add-f32@1` already provides. What this record adds to that ticket is not a new requirement but a value: the mask's fill is finite, and its attended entry is negative zero.

## What this record does not decide

- **Any numeric tolerance for the model-level comparison.** L1 already fixes that composing one from per-operation tolerances is the defect rather than the method, and [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) owns the bound.
- **Whether a fused attention program materializes its scores.** L4's, and it is a feasibility question at long `T` rather than a cost question at every `T`, per L1's 4.00 GiB bound.
- **The contraction that produces the scores.** [ADR 0087](../../decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) settles its identity and this record does not reopen it. The attention scale's position — on the score, not on an operand — is stated here as a graph fact because the softmax's input is defined by it, not as a claim about the contraction's signature.
- **Whether a decomposition capability is registered for the atomic families.** [`docs/ir.md`](../../ir.md) lists decomposition among the optional capabilities and L2 proposed declaring one for softmax and normalization. That remains a proposal; nothing here requires it, and admitting a key without one is a smaller first delivery.
- **GELU, in either form.** Not in this workload. A GPT-2-shaped diagnostic fixture would need one, and would need the erf-versus-tanh choice made explicitly rather than borrowed from here.

## Consequences for the ladder

**Inference.** L3′'s stated closure condition — every nonlinear, normalization, mask, and reduction requirement has a precise contract or a named unresolved decision, Metal feasibility boundaries are recorded, and justified delivery work has dependency-ordered tickets — is met by this record together with the two retained probes and the three tickets above. L4's trigger is "L3 and L3′ both deliver"; this delivers the L3′ half, and L3 remains gated on its own planning half.

**Inference — the honest maturity claim.** Nothing moved. Softmax, RMS normalization, and SiLU remain at R2, masking's `Select` alternative remains at R1, and the reductions beyond strict serial sum remain at R2. What L3′ delivers is a derivation record and two bounded measurements, to which the four-claim maturity vocabulary does not apply. Two of the measurements correct assumptions a competent implementer would otherwise have made — the mask's fill value and the softmax's reciprocal normalization — and that is the value of the rung, not a rung advance.
