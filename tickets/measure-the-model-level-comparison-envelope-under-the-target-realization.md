---
id: measure-the-model-level-comparison-envelope-under-the-target-realization
title: Measure the model-level comparison envelope under the target realization
status: done
priority: p1
dependencies: [retain-the-qwen-conformance-reference-logit-fixture]
related: [design-model-level-qualification-and-optimization, land-the-model-level-qualification-record, define-the-model-level-conformance-corpus, retain-the-c1-model-attribution-fixture]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, conformance, measurement, language-model, qwen, metal]
---
## User-visible outcome

The model-level comparison bound that [`define-first-metal-lm-workload`](define-first-metal-lm-workload.md) deliberately left `Unknown` becomes a measured quantity, derived from accepted contracts and measured target facts **before** any Tiler result exists — so the first Tiler execution is compared against a band nobody chose after seeing it.

## Evidence prerequisite

Run against [`docs/research/program-planning/first-metal-lm-workload.md`](../docs/research/program-planning/first-metal-lm-workload.md) and the L8 qualification record's *The bound: three named perturbations* section, not against remembered figures. The retained producer is [`spikes/program-planning/qwen3-conformance-fixture/`](../spikes/program-planning/qwen3-conformance-fixture/README.md); this ticket extends it rather than writing a second one, so that the perturbed passes belong to the same verified checkpoint, the same verified reference sources, and the same evaluated configuration as the retained F32 baseline.

## Required work

Extend the fixture's producer with a **joint** perturbed pass and retain its per-position deviation against the plain F32 pass. The three perturbations are applied together and the retained quantity is the joint deviation; three separately measured maxima are **not** the deliverable, and adding them would be the per-operation composition the workload profile's own bound method forbids.

- **P-reorder** — an independently legal F32 ordering. Already implemented as the fixture's `f64_unmodified` and `f64_promoted` variants; reuse them rather than adding a third ordering.
- **P-flush** — F32 subnormal inputs and results flushed to sign-preserving zero at every arithmetic site, which is what the qualified `apple9-f32-unified-msl4-macos26` row is measured to do and what the CPU reference does not.
- **P-elem** — each subordinate elementary function perturbed to the worst result its **registered** accuracy contract admits: `Ulp(tiler::ulp-reference-gap@1, 12)` for the exponential subordinate to `tiler::softmax-f32@1` and `tiler::silu-f32@1`, and `Faithful` for the reciprocal square root subordinate to `tiler::rms-norm-f32@1`. **The authority is the registered contract, not Table 8.1 directly** — the specification's 4-ULP exponential bound is stated under Apple's own ULP definition and crosses to Tiler's metric through the single registered `ScaledMetric` implication whose derivation carries a factor of three. A perturbation sized at 4 measures a bound Tiler does not claim.

### The P-flush mechanism must be proved, not assumed

The candidate mechanism is the host FPU's flush-to-zero mode — ARM `FPCR.FZ` on the Apple-silicon correctness host, reachable as `torch.set_flush_denormal`. **Its return value is not the verification.** Two positive controls are required in the same process, and both must be watched succeeding *and* failing:

- an elementwise expression whose exact F32 result is subnormal returns a sign-preserving zero with the mode on and the exact subnormal with it off;
- the same for a contraction that goes through the BLAS path, checked separately, because the elementwise and BLAS paths need not share the mode.

**Stop condition.** If either control fails, or if the mode's sign behaviour differs from the sign-preserving flush measured on the target row, P-flush is **not** established by this mechanism: record the exact failure, leave the term `Unknown`, and retain the two-term joint measurement with its gap stated. Approximating the term, or reporting the reordering envelope as if it covered the flush, is the defect this ticket exists to prevent.

## Retain

Under the fixture's existing conventions — full bytes regenerable outside version control, digests and bounded comparison values checked in:

- per position, the joint deviation over the whole vocabulary and restricted to the reference's own top-32 order, in both absolute and ULP terms;
- the greedy token under the joint perturbation at every position, and whether it agrees with the F32 baseline at all 18;
- **the comparison against the smallest runner-up gap, 0.266.** The exact-greedy gate the qualification record derives holds only while the measured band stays below that gap. Report the ratio, and if the band exceeds it, say so plainly rather than keeping the gate.
- the environment row and every verified digest, exactly as the existing record does;
- whether the P-flush controls passed, and under which mechanism.

## Explicit non-goals

No Tiler execution, no Metal compilation, no device. No B1-length row — the workload profile makes C1 the only fully retainable row and a B1 bound is a different measurement. No threshold: this produces the band, and the corpus and regression tickets decide what is gated on it.

## Reconsideration trigger

If the pinned checkpoint revision, the conformance prompt, the reference revision, the qualified target row's measured subnormal behaviour, or any of the three registered accuracy contracts is superseded, this measurement is re-derived rather than patched, and the superseding change says which retained rows survived.

## Closes when

The joint band is measured and retained beside the existing envelope, its relationship to the runner-up gap is stated, the P-flush mechanism is either proved by both controls or recorded as unestablished with the term left `Unknown`, and the workload profile's *What remains open* entry naming the comparison bound is updated to point at the retained result.

## Outcome

All four conditions are met. The band is retained in the existing record, [`spikes/program-planning/qwen3-conformance-fixture/results/2026-08-01-c1-conformance-attribution-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0/`](../spikes/program-planning/qwen3-conformance-fixture/README.md), as two new files beside the thirteen that were already there: `joint.tsv`, 72 rows, and `perturbation.tsv`, 60 keys.

**Measurement — the joint band.** Over all 18 positions and all four joint variants, the largest whole-vocabulary deviation from the plain F32 pass is **2.2101e-4** (`0.00022101402282714844`); restricted to the reference's own top-32 order it is **1.0872e-4**, or at most **87 ULP**. The band is set at position 0 by `joint_unmodified_alternating`. It is the deviation of a pass carrying P-reorder, P-flush and P-elem *together*; three separately measured maxima are not produced anywhere in the record, because adding them is the per-operation composition the region-accuracy contract forbids.

**Measurement — the greedy gate holds, and by a wide margin.** All 72 joint rows agree with the baseline's greedy token. The smallest runner-up gap is **0.2660789489746094** at position 10, the band's ratio against it is **8.3063e-4** — about **1,204×** below the gap — and the loosest position-by-position ratio is 1.52e-4 at position 1. The gate is kept, not abandoned, and the record carries the verdict as a derived value rather than an assertion: `verify_fixture.py` recomputes it from the band and the gap and refuses a flipped one.

**Measurement — P-flush is established, by the mechanism the ticket named, and it is the identity on this row.** `torch.set_flush_denormal` returns `true` for *both* directions on this host, so the return value decided nothing. Two positive controls in the same process did, each watched in both arms: an elementwise `float32 (-1e-38) * 0.01` returned `0x800116c2` with the mode off and `0x80000000` — a sign-preserving zero — with it on; a `[64, 2] @ [2, 64]` `torch.matmul` whose exact sum is `-2^-133` returned `0x80010000` off and `0x80000000` on. **The BLAS control's construction is the load-bearing part.** The obvious control — a negative subnormal *product* through a gemm — returns `+0` under the mode whether or not the flush preserved the sign, because `(−0) + (+0) = +0` by IEEE addition against a zero accumulator; that control cannot distinguish the two behaviours, and a `K = 64` zero-padded variant was watched doing exactly that. The retained control instead contracts two normal terms whose exact *sum* is the negative subnormal, with no subnormal input or intermediate, so the flush of the result is the only step that can set the sign. Sign behaviour therefore matches the target row's measured sign-preserving flush on both paths, the stop condition did not fire, and the term is carried.

**Measurement — and then the term's effect was measured rather than presumed.** The plain F32 pass re-evaluated with the mode in force is **bit-identical to the baseline at all 18 positions**, so no arithmetic site of this row produced or consumed an F32 subnormal. The joint band carries all three terms and the flush contributes nothing to it — which is a fact about this row's dynamic range, not about the mechanism: the weights supply none, the masked softmax entries underflow to exact zero rather than through the subnormal range, and no attended score, gate activation, or normalized state on this prompt reaches the `2^-126` floor. The carrier control agrees from the other side: the float64 joint pass is byte-identical with the mode off.

**Correction — 2026-08-02, on the reason the weights supply none.** This outcome previously gave it as "BF16 subnormals widen to F32 *normals*", which is false — that is binary16's behaviour, while BF16 shares binary32's exponent width and preserves the subnormal class, measured exhaustively at 254 of 254 in [the BF16 conversion record](../docs/research/numerics/bf16-computation-accumulator-and-conversion.md). The measured reason is a counted property of the pinned revision: 0 subnormal, 0 infinite, and 0 NaN stored values over all 596,049,920 elements of all 310 tensors, from [the corpus reachability probe](../spikes/program-planning/qwen3-corpus-reachability/README.md). **No measurement in this outcome changes** — the bit-identity, the two controls, the band, and the greedy gate were all measured rather than inferred from the removed clause — and the reconsideration trigger above gains a reading it did not have: a *checkpoint* revision carrying a BF16 subnormal makes P-flush a live term, not only a prompt with a wider dynamic range.

**Measurement — P-elem, sized from the registered contract and not from Table 8.1.** The exponential subordinate to `tiler::softmax-f32@1` and `tiler::silu-f32@1` is moved 12 ULP under `tiler::ulp-reference-gap@1`, and the reciprocal square root subordinate to `tiler::rms-norm-f32@1` by one ULP, the supremum of the `Faithful` band. Sizing the exponential at 4 would have measured a bound Tiler does not claim; `verify_fixture.py` now refuses a record that does, and that refusal was watched firing. The perturbation widens the reordering envelope from 2.048e-4 to 2.2101e-4, about **8%**, so reduction order remains the dominant term on this row. The `elem_zero` control — `joint_unmodified_outward` at zero ULPs — reaches exactly `0.0002048015594482422`, the `f64_unmodified` figure to the last bit, so the expanded softmax and SiLU spellings the perturbation needed contribute nothing of their own, and it is compared against the variant it controls rather than against the band, so a perturbation that reached nothing in that variant could not hide behind another's deviation.

**Measurement — the two sign policies differ by almost an order of magnitude, and that is why both were run.** Against the same control, `joint_unmodified_outward` reaches `0.0002067089080810547`, a **0.93%** move, while `joint_unmodified_alternating` reaches 2.2101e-4, a **7.9%** move. **Inference.** A uniform relative perturbation of every exponential very nearly cancels in a quotient by their own sum, so the correlated policy reaches the model boundary mostly through the SiLU gate and the RMS scale, while a sign taken from the result's own low mantissa bit does not cancel in the softmax. A record that had run only the correlated policy would have measured P-elem's contribution at roughly an eighth of its size and reported a band of 2.067e-4, and would have looked like a clean measurement while doing it.

**What the band is not, stated where a reader will act on it.** Two sign policies are retained — `outward`, worst where a perturbation propagates through a sum and exactly cancelling inside a softmax normalization, and `alternating`, taken from the result's own low mantissa bit and therefore not cancelling there — and the band is the maximum over both. Neither is a search over the 2^N per-element sign assignments, which is combinatorial and per output, so **the true worst case within these registered contracts is at least the measured band and is not bounded above by it.** That sentence is in `perturbation.tsv`, the spike README, and the workload profile, because a reader who mistook the band for a proven upper bound would treat a legal realization outside it as a defect.

**Every new check was watched refusing.** Twelve deliberate one-value perturbations of a scratch copy of the record, each with `manifest.tsv` consistently re-hashed so the cross-file check under test is what fires rather than the digest: a band raised above its own rows, a greedy token moved off the baseline's, a gap moved off `positions.tsv`, the exponential resized to 4, the BLAS control's mode-on arm reporting the unflushed subnormal, the reachability count moved off its population, the gate verdict flipped, the zero-magnitude control raised above the variant it controls, that control's variant maximum moved off the variant's own rows, that control naming a variant `joint.tsv` does not carry, the agreement count moved off `joint.tsv`, and `environment.tsv` resized away from `perturbation.tsv`. All twelve refused at exit 5; the unperturbed copy passed in the same matrix. Six producer guards were driven to their stop with the input each exists to refuse — a moved gap minimum, a joint population one variant short, an unknown sign policy, an elementary result at the F32 range limit, a non-SiLU activation, and a variant that produced 17 positions — each exiting 4 with its positive arm watched passing.

**Reproducibility, and that the extension did not perturb the pass it measures.** Every previously retained file except `environment.tsv`, which gained five `joint.*` keys and removed none, and `manifest.tsv` regenerated byte-identically. `produce_fixture.py --compare` then regenerated the whole production and all **15** retained files matched byte for byte, so the joint passes and the flush mode are as deterministic on this host as what they were added beside. `verify_fixture.py` runs **47,646** counted checks needing no model and names every population it examined.

**Environment.** Apple M4 Max, macOS 27.0 build 26A5388g, arm64, 36 GiB; Python 3.11.12, `torch` 2.6.0, `transformers` 4.51.0, `numpy` 2.2.5, `torch.set_num_threads(1)`. The full row and every verified checkpoint and reference-source digest are in `environment.tsv`, unchanged.

**Measurement boundary.** One host, one checkpoint revision, one reference revision, one prompt, 18 positions, batch 1, greedy, F32. Nothing here generalizes to a B1-length row, another prompt, another checkpoint, or the quantized path. No Tiler execution, no Metal compilation, no device.

**For the carrier of the L8 record.** The drafted *The bound: three named perturbations* span inside [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) was this measurement's protocol authority and was deliberately **not** edited, so its byte-identical transfer stays intact. Its two open conditionals are now settled and [`land-the-model-level-qualification-record`](land-the-model-level-qualification-record.md) is where that lands: the bound is 2.2101e-4 rather than a proposal, and the gate's condition — that the band stay below the smallest runner-up gap — is measured to hold with three orders of magnitude to spare.

**Filed rather than absorbed.** [`search-the-p-elem-sign-assignment-for-the-model-level-band`](search-the-p-elem-sign-assignment-for-the-model-level-band.md), deferred with its activation trigger, because the gap between a full-magnitude sample and a searched worst case only matters once a Tiler result lands near the band.
