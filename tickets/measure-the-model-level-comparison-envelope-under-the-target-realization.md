---
id: measure-the-model-level-comparison-envelope-under-the-target-realization
title: Measure the model-level comparison envelope under the target realization
status: in-progress
priority: p1
dependencies: [retain-the-qwen-conformance-reference-logit-fixture]
related: [design-model-level-qualification-and-optimization, land-the-model-level-qualification-record, define-the-model-level-conformance-corpus, retain-the-c1-model-attribution-fixture]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, conformance, measurement, language-model, qwen, metal]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785626991
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
