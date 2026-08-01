---
id: correct-the-softmax-worked-example-and-its-recorded-divergence
title: Correct the softmax worked example and record its reciprocal divergence
status: in-progress
priority: p1
dependencies: []
related: [admit-the-softmax-family, scope-transformer-nonlinear-normalization-and-reductions, design-model-level-qualification-and-optimization, retain-the-c1-attention-block-conformance-evidence]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, softmax, measurement, correction, transformer]
claimed_from: todo
assignee: worker-softmax-record
lease_expires_at: 1785602431
---
## User-visible outcome

A reader of the L3′ derivation's softmax worked example is told which implementation produced its bit patterns, so the example stops reading as a demonstration that the pinned formula reproduces the reference — which, at that row, it does not.

## Why this is filed

**Fact — the record's own numbers do not follow from its own formula.** [Transformer non-linear, normalization, and reduction contracts](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) states the softmax formula as `r_i = e_i * (1 / d)` and then gives a worked example over `[1.0, 2.0, 3.0, mask]` whose recorded intermediates are `e = 0x3e0a9555 0x3ebc5ab2 0x3f800000 0x00000000` and `d = 0x3fc06957`, and whose recorded outputs are `0x3db861f2 0x3e7a9a18 0x3f2a4d3a 0x00000000` summing to `0x3f7ffffe`. Applying the record's own formula to the record's own `e` and `d` gives `0x3db861f3 0x3e7a9a1a 0x3f2a4d3b 0x00000000`, summing to exactly `0x3f800000`. The recorded outputs require a reciprocal of `0x3f2a4d3a`, while the correctly rounded `1.0 / d` is `0x3f2a4d3b`.

**Measurement — 2026-08-01, in the retained probe's own pinned environment** (`torch` 2.6.0, `transformers` 4.51.0, CPU, F32, run from `spikes/numerics/transformer_reference_semantics/` with `uv run --offline`):

- `torch.nn.functional.softmax` on that row returns the record's bits, so the record's *observation* is correct.
- Computed from the reference's own `e` and `d`, **both** `e * (1/d)` and `e / d` return `0x3db861f3 0x3e7a9a1a 0x3f2a4d3b`. So the reference's fused kernel matches neither of the two spellings the record names, at the row the record uses to illustrate them.
- A single constant explains every finite output: dividing each recorded output by its exponential yields `0x3f2a4d3a` at all three positions. **The divergence is in the reciprocal**, not in the exponential and not in the sum, both of which the reference reproduces bit for bit.
- Over 20,000 random rows per width, `F.softmax` equals `e * f32(1/d)` at **every** element at width two (40,000 elements) and width three (60,000 elements), and diverges from *both* spellings by up to four ULP from width four upward.

**Inference — this is the same class of finding as the `rsqrt` one, and it lands in the same place.** The reference model performs an approximation its own formula distinguishes, and only a discriminating argument detects it. It is a finding *about the reference model*, feeding a model-level bound rather than a per-operation tolerance, exactly as the correction under *RMS normalization* records for `torch.rsqrt`.

**Inference — the probe's attribution of `matches_neither` is at least incomplete.** [The reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md) explains its `matches_neither` column as "the denominator's own accumulation order disagreeing with the naive sum". At the worked example the denominator agrees exactly with the strict left fold, so accumulation order cannot be the cause there. The approximate reciprocal is a second, unrecorded source, and the probe's prose currently reads as if there is one.

## Required delivery

- **Correct the worked example's presentation** in the L3′ record: keep the measured bits, label them as `torch.nn.functional.softmax`'s output at that row, and state beside them what the pinned formula gives and why the two differ. The current table reads as a derivation of the formula's own result.
- **Correct the row-sum claim's evidence.** "The outputs sum to `0x3f7ffffe`, not to `0x3f800000`" is true of the reference at that row and *false* of the pinned formula there. The claim it supports — softmax does not produce a row summing to exactly one — is still true, and `admit-the-softmax-family` pins two rows that carry it under the pinned formula: `[0.0, 2.0]` sums to `0x3f7fffff` and `[0.0, 1.0, 0.0]` to `0x3f800001`, both at widths where the reference and the pinned formula agree at every element.
- **Correct the probe's `matches_neither` explanation** to name both sources, or measure which of them dominates at each width.
- **Decide whether the probe gains rows.** Two are missing and each was needed by the admission and had to be measured outside the retained record: a worked-example row, and a softmax row with a NaN score. The exact check for the second: `grep -i nan spikes/numerics/transformer_reference_semantics/results/*/record.tsv` returns only `silu_inputs` and the SiLU result rows. Adding them makes the D-2 evidence and the divergence reproducible from the retained record instead of from a re-run.
- **State the boundary.** The measurements above are one host class, CPU, F32, and those two package versions. Whether the divergence is the CPU vectorized path, a NEON reciprocal estimate, or something else is *not* established, and this ticket must not assert a mechanism it did not measure.

## Non-goals

Changing `tiler::softmax-f32@1`'s pinned formula. The width-two and width-three agreement is what selects the reciprocal form, and it is stronger evidence than the record originally carried: agreement at every element rather than at discriminating elements only. Reproducing `torch`'s approximate reciprocal is likewise a non-goal — the registered contract states what the operation means, and the reference model falling outside it is recorded rather than adopted.

## Reconsideration trigger

Active now: the record is cited by an implemented family whose conformance corpus disagrees with it at a row the record presents as agreeing.
