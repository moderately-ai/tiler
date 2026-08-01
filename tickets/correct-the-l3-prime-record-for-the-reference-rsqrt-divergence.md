---
id: correct-the-l3-prime-record-for-the-reference-rsqrt-divergence
title: Correct the L3-prime record for the reference rsqrt divergence
status: todo
priority: p1
dependencies: []
related: [admit-the-rms-normalization-family, implement-parallel-reduction-strategies, design-model-level-qualification-and-optimization]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, normalization, transcendental, correction]
---
## User-visible outcome

The L3′ derivation records what its own retained measurement of `torch.rsqrt` actually shows, and decision **D-3**'s entry records that it closed — so the next reader of that record is not told an open question is open, or that a measured value is the reference.

## The correction, and how to reproduce it

**Measurement — reproduced, not asserted.** The [reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md) records `rsqrt_of_eps_alone` as `0x4479ffff`, from `torch.rsqrt(torch.tensor(1e-6, dtype=float32))`. The argument is the binary32 rounding of `1e-06`, payload `0x358637bd`, whose exact value is `9.999999974752427e-07`. The exact reciprocal square root of that value is `1000.00000126237864845…`, so the two binary32 values bracketing it are `0x447a0000` (exactly `1000.0`) and `0x447a0001`, and the correctly rounded value is `0x447a0000`.

`0x4479ffff` is one step *below* that pair — about `1.02` ULP from the exact reference — so it is not correctly rounded and not faithful either. It is exactly what the two-rounding composition `f32(1 / f32(sqrt(t)))` delivers at this argument, which is the spelling the derivation's own "`rsqrt`, not `1 / sqrt`" sentence exists to exclude.

The reproduction is three lines of exact arithmetic and is checked in Rust at `crates/tiler-reference/src/rms_norm/tests.rs::the_certified_reciprocal_square_root_separates_rsqrt_from_one_over_sqrt`, which asserts both values and their difference.

**Consequence — it propagates to a second recorded row.** The probe's `rms_subnormal_vector` is `0x02081cb9`. The squares of `1e-40` underflow to exactly `+0.0`, so that row's reciprocal square root argument is `eps` alone; with the correctly rounded scale the row is `0x02081cba`. The one-step difference is entirely the `rsqrt`.

## What the record should say, and what it should not

The derivation's RMS normalization table currently reads the zero row's measurement as if it were the reference: "`rsqrt(0 + 1e-6)` is `0x4479ffff` (≈ 999.99994, not 1000)". That sentence is a correct *measurement of one implementation* and is being read as the normative value. It needs the boundary restated rather than the number changed — the measurement stands, its class does not.

**Non-goal — do not change the pinned formula.** Nothing here suggests Tiler should reproduce `torch.rsqrt` bit for bit. `tiler::rms-norm-f32@1` states a `Faithful` contract derived from Metal's Table 8.1 and §8.2, and the reference model's value falls outside it; that is a finding about the reference model, and the model-level bound it feeds is [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md)'s, not this ticket's.

## Also owed by this ticket

- **D-3 closed.** `admit-the-rms-normalization-family` settled it as *define, not refuse*, with the elimination stated and [`scope-a-value-domain-precondition-for-squaring-overflow`](scope-a-value-domain-precondition-for-squaring-overflow.md) carrying the deferred capability. The derivation's "Unresolved decisions" entry still says it closes when the key is admitted; the key is admitted.
- **D-4's `rsqrt` half.** The record says every Table 8.1 entry the three verticals need is unadoptable without a derivation. The reciprocal square root's derivation now exists: correctly rounded under either mode §8.2 admits is exactly the faithful pair, so the entry is adoptable as `AccuracyContractForm::Faithful` and needs no metric reconciliation at all. Gap 1 and Gap 4 bind disjoint halves and only Gap 1's half needed a registered implication.
- **The support-matrix cross-reference.** `docs/roadmap.md` already carries the normalization's own row; the derivation's "Consequences for the ladder" section still says nothing moved.

## Closes when

The derivation states the measurement's class correctly, records D-3's answer and its elimination, records what D-4's `rsqrt` half now supports, and the `informs`/catalog metadata still agrees with the corpus.
