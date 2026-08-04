---
id: fuse-quantized-weight-decode-into-the-strict-contraction
title: Fuse the quantized weight decode into the strict contraction
status: todo
priority: p2
dependencies: [widen-the-physical-vocabulary-for-per-axis-quantized-component-access, implement-workload-selected-quantized-parameter-maps, realize-the-tiled-contraction-schedule-and-its-metal-emission, reclassify-language-model-work-as-a-conformance-track]
related: [implement-first-quantized-backend-profile, admit-reassociated-contraction-schedule-alternatives, calibrate-device-cost-models, scope-first-quantized-lm-profile]
scopes: [implementation/compiler, implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, contraction, fusion, metal, language-model]
---
## User-visible outcome

A `Dequantize` feeding a contraction becomes an operand *access* of the surviving tiled realization rather than a materialized `f32` tensor, so the kernel reads the quantized bytes. The materializing plan stays a retained alternative and the two are separately costed, because which one wins is a measurement this ticket does not have.

## Why fusion is the whole point, from measured bytes

**Fact.** [The contraction realization record](../docs/research/scheduling/first-metal-contraction-realizations.md) measured the complete decode vocabulary projection reading a 622,207,744-byte `f32` weight per dispatch at 4,247 µs — 146 GB/s — and concluded the cell is bandwidth-bound and "no arithmetic schedule can move it".

**Inference — an explicit materializing boundary loses in both directions**, as [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) works through. Materializing per dispatch reads about 156 MB of codes and parameters, writes 622 MB of `f32`, and reads it back: roughly 1.40 GB of traffic against the `f32` baseline's 0.622 GB, so a quantized decode would be at least 2.2× slower than the baseline it is meant to improve. Materializing once and retaining the result puts the 622 MB of `f32` weights back in residency and the memory win is exactly zero.

**Inference — and the fusion consumes no numerical permission.** A per-output-channel scale is loop-invariant over the contracted index, so the fused contributor sequence, its ascending order, and its unseeded first-product start are all unchanged; only each contributor's value changes, which is what "a different operand" means. Reassociation, permutation, and distributivity stay unconsumed — including the integer-domain factorization `s[o] · (Σ a·w − z[o]·Σ a)`, which is a distributivity rewrite that no contract Tiler can express grants.

## Implementation keys

- The semantic graph keeps stating `Dequantize` then `Contraction`. Do not introduce a packed-contraction operation: that densifies a physical choice into the logical graph, and the architectural contract forbids it.
- Fusion legality must be decided by the existing authority, with the decode's numerical requirements composed into the region's rather than assumed compatible. A fused region whose realization the target cannot honour is rejected with an explainable reason, not costed.
- **The fused and materialized forms must be proved bit-identical for this profile, and the proof must say why it is profile-specific.** The decode's compute type and expressed type are both `f32`, so eliding the store rounds nothing away; a profile whose compute type were wider than its expressed type would delete a real materialization rounding point and the same fusion would then be a semantic change. Encode that condition as a check, not as a comment.
- Emission adds a subtract, a convert, and a multiply to the contraction's inner statement. `-ffp-contract=off` governs statements the emitter writes separately and says nothing about an instruction whose fusion is the instruction, so the per-statement emission rule is what holds the line — assert on the emitted module that the accumulation path carries no fused multiply-add, exactly as the strict contraction already does.
- Retain the materializing alternative. Selection between them is the cost model's, and until [`calibrate-device-cost-models`](calibrate-device-cost-models.md) supplies calibrated evidence neither may be described as the faster one.
- Prefill and decode may legitimately select differently: prefill is compute-bound in the retained measurements and decode is bandwidth-bound. Do not force one plan across both.

## Closes when

One weighted projection of the workload's index structure compiles with the decode fused into its weight operand access, its result is bit-identical to the materializing plan and to the reference at the profile's own cells, the materializing plan is retained as a costed alternative, a fused region whose numerical requirements the target cannot honour is refused with a reason that was watched firing, the emitted module carries no fused multiply-add on the accumulation path, targeted package tests and Clippy pass, `tkt lint` and `git diff --check` pass, and one `make full` passes.

## Graph maintenance

- Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) from [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md).
- Nothing here admits a per-block scheme. A block map's fused form partitions the contracted axis into contiguous intervals merged in order, which consumes reassociation; [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) owns that alternative and a caller granting the permission is what would reopen it.
- No device-optimal claim may be made from this ticket. Experiment E-2 of the record and calibrated costs are separate prerequisites.
