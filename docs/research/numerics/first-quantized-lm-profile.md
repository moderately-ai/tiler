---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.first-quantized-lm-profile"
kind: "research"
title: "First quantized language-model profile"
topics: ["numerics", "quantization", "dtypes", "language-model", "contraction", "metal", "memory", "qwen"]
catalog_group: "dtypes-quantization"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["bounded-measurement", "exhaustive-finite", "primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.correctness-and-testing"]
depends_on: ["tiler.research.program-planning.first-metal-lm-workload", "tiler.research.scheduling.first-metal-contraction-realizations", "tiler.research.numerics.affine-quantization-semantics", "tiler.research.numerics.quantized-value-and-transform-contract", "tiler.research.apple-targets.numerical-behaviour"]
ticket: "scope-first-quantized-lm-profile"
---

# First quantized language-model profile

**Status:** durable selection and elimination record for rung L7 of the language-model inference ladder. It is a research outcome, not a capability: nothing here registers a scheme, admits a parameter map, installs a lowering capability, or moves any cell of the [dtype support ledger](../../dtype-support.md). What it delivers is one selected profile, the elimination that produced it with the ground stated per candidate, two measured stages, one exhaustive-finite derivation about target honourability, and the dependency-ordered delivery graph.

## Why this record lives in `research/numerics`

**Inference.** Its subject is which *quantized value scheme* survives — code type, expressed type, parameter map, grouping axis, conversion identity — which the `dtypes-quantization` catalog group already owns beside [affine quantization numerical semantics](affine-quantization-semantics.md) and [the quantized value and transformation contract](quantized-value-and-transform-contract.md). It is not workload-profile shaped: [L1](../program-planning/first-metal-lm-workload.md) already fixed the model, the rows, and the memory arithmetic, and this record consumes them rather than restating them. The one physical result it derives — that a per-block map cannot be contracted under the governed contract — is applied from [L3](../scheduling/first-metal-contraction-realizations.md)'s measured elimination rather than newly measured here.

## Traceability

- **Work record:** [`scope-first-quantized-lm-profile`](../../../tickets/scope-first-quantized-lm-profile.md).
- **Ladder position:** rung L7 of [the roadmap's language-model ladder](../../roadmap.md#the-ladder). Its trigger is "L1 and L3 deliver and milestone 2Q supplies the quantized-value proof"; all three fired.
- **Consumed authorities, read as evidence and not edited:** [the workload profile](../program-planning/first-metal-lm-workload.md) for the pinned checkpoint, the F32 memory arithmetic, the C1 and B1 rows, the qualified target row, and the measured F32 sensitivity envelope; [the contraction realization record](../scheduling/first-metal-contraction-realizations.md) for the surviving strict realization, the six-candidate elimination, the permission each split consumes, and the two measured host rows; [`prototype-quantized-value-vertical`](../../../tickets/prototype-quantized-value-vertical.md) for the answer that a quantized value is a typed compound contract and for the delivered strict-affine U4/U8 verticals; [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) for the qualified row's subnormal flush and for what is unmeasured; [ADRs 0029 through 0033](../../decisions/0029-affine-quantization-parameter-maps.md) for parameter maps, first-class encoded values, strict affine evaluation, and the semantic-validation/physical-enforcement split.
- **Retained experiment:** [Qwen3-0.6B-Base candidate quantization profile probe](../../../spikes/numerics/qwen3-weight-quantization-profiles/README.md), with two result records — weight-space error over all 197 candidate tensors, and the model-visible C1 observable over sixteen profile runs.

Claims are labelled **Fact** when traced to inspected source, primary documentation, or a merged record, **Inference** when derived from stated facts, **Measurement** when tied to an exact environment and procedure, and **Proposal** when not yet accepted or tested.

## What this rung decides, and what it must not be

**Fact — the checkpoint is not quantized, so a quantized profile is a new artifact.** [L1](../program-planning/first-metal-lm-workload.md) read the safetensors header at the pinned revision: 310 tensors, **every one BF16**, and the workload widens them to F32. There is no quantized checkpoint to be faithful to, no vendor scheme whose bit layout constrains the choice, and no external format compatibility obligation. **Inference.** The profile is therefore chosen against the workload's own memory and accuracy behaviour and against what Tiler and the measured target can actually realize — which is exactly what this ticket forbids replacing with a format picked for its reputation.

**Fact — three claims are distinct and this record keeps them apart throughout.** [The governed dtype catalog](../../../crates/tiler-ir/src/semantic/catalog.rs) registers a *recognized identity* for all six OCP FP8/FP6/FP4 formats, E8M0FNU, and six OCP MX schemes; a *statable contract* is a `ResolvedValueType` a registry will validate; an *executable* profile has a lowering, an emission, and a device. The ledger's own note says it: for MX, "**No MX value can be constructed**". A candidate that is registered but not statable is not a weaker candidate — it is not a candidate at all until something makes it statable, and saying so is the difference between an elimination and a preference.

**Fact — a smaller artifact is the least interesting property on the table, and the control proves it.** The checkpoint's own BF16 weights round-trip through F32 exactly, so replacing all 197 tensors with their BF16 round trip is *bit-identical* at every C1 position — **Measurement:** maximum logit deviation `0.000000e+00` over 18 positions — at 0.500 of the F32 weight bytes. **Inference.** Any quantized candidate has to be judged against 1,192,230,912 bytes and zero error, not against F32's 2,384,199,680 bytes; a candidate that halves the weights and changes the model is losing to a control that halves the weights and does not. Every table below carries BF16 as the control row for that reason. That control is *not* free elsewhere — BF16 has no semantic operation, no reference evaluator, and no physical carrier, and [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) measured both that BF16 arithmetic flushes and that the iOS simulator refuses BF16 pipeline creation — but as an ingestion storage width widened to F32 before any arithmetic it needs none of those. [`spike-bf16-through-the-second-dtype-seams`](../../../tickets/spike-bf16-through-the-second-dtype-seams.md) owns it and this record files nothing against it.

## The candidate space

**Fact — the representations that could be named at all**, with what each would require before it could be evaluated. The middle column is the strongest of the three claims above that the candidate currently holds.

| Candidate | Strongest current claim | What it would take to reach the next claim |
| --- | --- | --- |
| BF16 storage, widened at ingestion (control) | recognized identity (`tiler::bf16@1`) | no scheme, map, or zero point; a storage width and an ingestion conversion |
| Strict-affine U4/F32, per tensor | **executable** target-neutral vertical; refused by the measured Metal profile | a target that honours its declared subnormal preservation |
| Strict-affine U8/F32, per tensor | statable contract; reference-tested; no physical vertical at all | schedule, kernel, program, artifact, Metal, runtime |
| Strict-affine i4/i8, other widths, other expressed types | architectural seam | a selected profile naming code, expressed, scale, and compute types |
| Strict-affine, per-axis or per-block map | architectural seam (`ParameterIndexMap` exists; only `per_tensor()` is constructible) | [`implement-workload-selected-quantized-parameter-maps`](../../../tickets/implement-workload-selected-quantized-parameter-maps.md) |
| OCP MX (MXFP4/6/8, MXINT8) | recognized scheme identity only | a non-per-tensor map first; the only map that exists is per-tensor, which is the wrong association for a 32-element block |
| OCP FP8 E4M3FN / E5M2 as weight storage | recognized scalar identity only | a scalar-dtype vertical, not a quantization profile; `MetalFloatArithmeticType` names exactly F32, F16, Bf16 |
| Codebook, hierarchical-scale, mask/outlier, NVFP, GGML | type-system reservation | an ordered role set, a producer, and a consumer |

**Inference — the MX and FP8 rows are eliminated before any accuracy or performance question is asked**, and for a reason that is structural rather than numerical: no value of those schemes can be constructed, so there is nothing to measure, nothing to lower, and no artifact whose identity could carry them. That is hard feasibility, not cost, and this record keeps it on that side of the line. Their reconsideration triggers are unchanged and live in [the dtype support ledger](../../dtype-support.md#other-affine-and-ocp-mx-schemes).

**Inference — the codebook and vendor rows are eliminated the same way and more strongly**: they have no role set at all, so they cannot even be spelled.

That leaves the strict-affine family, whose scheme, conversion contract, refusals, reference evaluation, and identity behaviour [`prototype-quantized-value-vertical`](../../../tickets/prototype-quantized-value-vertical.md) already delivered. The rest of this record eliminates *within* it, on three axes taken in order: measured model quality, contraction legality, and target honourability.

## Elimination axis 1 — measured model quality

**Measurement — weight-space reconstruction error and exact byte cost, over all 197 candidate tensors and 595,984,384 parameters** of the pinned checkpoint. Codes are packed at the declared width; each group carries one F32 scale and one code-width zero point. The conversion is exactly what `tiler::strict-affine@1` registers; calibration is asymmetric min-max with `+0.0` representable, which is an ingestion-side choice and is measured separately below.

| Profile | Relative Frobenius error | Stored bytes | Bits per element | Smallest scale |
| --- | --- | --- | --- | --- |
| per-tensor U4 | 4.968e-1 | 297,993,177 | 4.000 | 1.908e-2 |
| per-tensor U8 | 3.149e-2 | 595,985,369 | 8.000 | 1.122e-3 |
| per-channel U4 | 1.398e-1 | 300,224,192 | 4.030 | 3.998e-4 |
| **per-channel U8** | **8.231e-3** | **598,464,384** | **8.033** | **2.352e-5** |
| per-group32 U4 | 8.158e-2 | 381,802,496 | 5.125 | 1.307e-4 |
| per-group64 U4 | 9.244e-2 | 339,897,344 | 4.563 | 2.029e-4 |
| per-group128 U4 | 1.028e-1 | 318,944,768 | 4.281 | 2.309e-4 |
| per-group128 U8 | 6.048e-3 | 619,265,024 | 8.313 | 1.358e-5 |

**Measurement — the model-visible observable on the workload's own C1 row.** Each profile replaces the weighted projections (and, in the second variant, the tied embedding matrix), and the row's fixed 10-token prompt and 8-step greedy decode are re-run and compared against an F32 baseline computed in the same process. `Sequence` is whether the 18-token C1 sequence is reproduced exactly; `Greedy` is per-position argmax agreement across all 18 positions.

| Profile | Model weight bytes | vs F32 | Sequence | Greedy (proj. only) | Greedy (+ embedding) | Median logit deviation |
| --- | --- | --- | --- | --- | --- | --- |
| BF16 control | 1,192,230,912 | 0.500 | yes | 18/18 | 18/18 | 0.000e+0 |
| per-tensor U4 | 298,255,321 | 0.125 | **no** | 0/18 | 0/18 | 5.04e+0 |
| per-tensor U8 | 596,247,513 | 0.250 | **no** | 9/18 | 8/18 | 7.67e-1 |
| per-channel U4 | 300,486,336 | 0.126 | **no** with embedding | 15/18 | 6/18 | 1.25e+0 |
| **per-channel U8** | **598,726,528** | **0.251** | **yes** | **17/18** | **17/18** | **1.08e-1** |
| per-group32 U4 | 382,064,640 | 0.160 | **no** | 7/18 | 7/18 | 1.31e+0 |
| per-group128 U4 | 319,206,912 | 0.134 | **no** | 7/18 | 8/18 | 1.38e+0 |
| per-group128 U8 | 619,527,168 | 0.260 | yes | 18/18 | 18/18 | 5.61e-2 |

**Inference — every U4 candidate is eliminated on measured model quality, including the one Tiler can already execute.** Per-tensor U4 is the only quantized profile with a delivered target-neutral vertical — role-addressed accesses, packed extraction, widened subtraction, verified kernel, artifact identity, decoded views — and it agrees with the F32 baseline's greedy token at *zero* of eighteen positions. That is the shape of elimination this ticket asked for: the cheapest and most nearly built candidate is discarded by evidence rather than preserved because it was nearly built.

**Inference — token-sequence equality is not the criterion, demonstrated rather than asserted.** Per-channel U4 over the projections reproduces the C1 18-token sequence exactly while disagreeing with the baseline's argmax at three of eighteen positions. L1's oracle already stated that materially wrong logits can retain the same argmax; this is a measured instance of it inside the candidate set, and it is why the table reports both columns.

**Measurement — the ordering survives a different calibration, and calibration is not what decides the widths.** Sweeping exact min-max against two-sided 99.9% and 99% clipping over one complete decoder layer plus the embedding:

| Profile | min-max | clip 99.9% | clip 99% |
| --- | --- | --- | --- |
| per-tensor U4 | 4.736e-1 | 1.646e-1 | 1.423e-1 |
| per-channel U4 | 1.404e-1 | 1.311e-1 | 1.171e-1 |
| per-group128 U4 | 1.027e-1 | 1.018e-1 | 1.021e-1 |
| per-group32 U4 | 8.156e-2 | 8.132e-2 | 8.213e-2 |
| per-tensor U8 | 2.866e-2 | 3.759e-2 | 8.908e-2 |
| per-channel U8 | 8.263e-3 | 1.541e-2 | 5.691e-2 |
| per-group128 U8 | 6.044e-3 | 7.299e-3 | 4.209e-2 |
| per-group32 U8 | 4.800e-3 | 5.315e-3 | 2.365e-2 |

**Inference — three things follow, and only the first is obvious.** Within each width, error decreases monotonically from per-tensor to per-channel to per-group128 to per-group32 under *every* calibration, so the granularity ordering is a structural property and not an artefact of the default. Clipping rescues coarse U4 substantially and hurts U8 at every granularity, because at eight bits the distribution tail is worth resolving and at four bits it is not — so there is no single calibration that is best for both widths, and a cross-width comparison is calibration-dependent in a way the within-width comparison is not. And per-tensor U4's *best* measured calibration, 1.423e-1, is still worse than per-group32 U4's *worst*, which is why no calibration work would reopen it.

**Measurement boundary — what this axis does not establish.** One prompt, one checkpoint, eighteen positions, batch 1, greedy, weights only, three calibrations. A profile that agrees at all eighteen C1 positions has not been shown to agree at a nineteenth, on a B1 row, or on another prompt; the retained record states this and the accuracy criterion below is written so that it does not depend on it.

## Elimination axis 2 — contraction legality, which is where per-block dies

This is the decisive axis and it is not an accuracy argument.

**Fact — the workload's weighted projections are contraction index structure 1, `td,od->to`.** [L3](../scheduling/first-metal-contraction-realizations.md) resolved 197 of the 253 contraction occurrences into it, with operand 1 the weight at `(o, d)`, the free index `o` the output feature, and `d` the contracted index. Every checkpoint weight is stored `[D_out, D_in]`, so a weight's axis 0 is `o` and its axis 1 is `d`.

**Inference — a per-output-channel parameter is constant along the contracted axis, and a per-block parameter is not.** A per-channel scale `s[o]` and zero point `z[o]` are loop-invariant over `d`, so the contraction's contributor at `d` is `a[t,d] * fl(fl(f32(i32(w[o,d]) - i32(z[o]))) * s[o])`, and the fold over `d` is unchanged: same contributors, same ascending order, same seed at the first product. A per-block scale `s[o, d/32]` changes inside the fold, so summing the block's contributions and scaling them — the form that makes a block scheme worth having — partitions the contracted axis into contiguous intervals and merges them in order.

**Fact — that partition is exactly the topology L3 measured and named.** Its `ksplit_contiguous` candidate "partitions the contracted axis into contiguous intervals merged in order, which consumes reassociation alone", was attributed uniquely to `contiguous_split+ftz` against twenty-two exactly evaluated topologies, and its elimination row reads: "**Only under reassociation** … No contract registered for this workload grants it yet."

**Inference — every per-block affine candidate is therefore inadmissible under the governed contract for this workload, and per-channel is admissible consuming no permission.** This is a hard-feasibility result, not a cost comparison: the block scheme's fused form computes a different reduction topology, and [Numerical semantics](../../numerical-semantics.md) forbids treating a contract as a search dimension while [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) forbids any authority from substituting a realization to make a plan feasible. Per-group128 U8's better measured accuracy — 18/18 against per-channel U8's 17/18 — does not survive that, because an inadmissible plan is rejected before it is costed and is not offered as a faster alternative.

**Inference — the block route is deferred with an exact trigger, not refused forever.** A caller that grants reassociation for this workload makes `ksplit_contiguous` legal and makes per-block affine a candidate again, at 6.05e-3 error and 0.260 of the F32 weight bytes. That is a contract-selection decision for a caller, and [`admit-reassociated-contraction-schedule-alternatives`](../../../tickets/admit-reassociated-contraction-schedule-alternatives.md) already owns the schedule alternative it would need. Nothing here should be read as evidence against block schemes in general.

**Fact — the block-size question that would otherwise have bitten does not arise.** [L3](../scheduling/first-metal-contraction-realizations.md) records that padding a ragged contracted extent owes a neutrality proof, "because `+0.0` is the strict sum's empty result and is *not* its bitwise-neutral padding", and warns that "a workload with a non-multiple head dimension or a quantized group size would trigger the obligation immediately". Every contracted extent in the workload is 1024, 2048, or 3072, each a multiple of 32, 64, and 128, so no measured group size would have padded. **Inference.** That is a property of this workload and this profile; it is not a reason to believe a future group size is safe.

## Elimination axis 3 — target honourability, and an obligation that is stronger than it needs to be

**Fact — the admitted strict-affine contract is currently refused by the measured Metal profile, and correctly so.** The registered decode contract declares `preserve-subnormals` unconditionally, the qualified `apple9-f32-unified-msl4-macos26` row flushes F32 input and result subnormals to sign-preserving zero, and `tiler-metal` emits the structured U4 dequantization vocabulary and then refuses with `SubnormalFlushInArithmetic` before producing a falsely executable payload. **Inference.** That refusal is the fail-closed behaviour working; it is not a defect and must not be removed by weakening the contract.

**Fact — the decode's exact evaluation, from the registered contract.** `ENCODED_NUMERIC_DECODE_EVALUATION` is `widen-code-and-zero-point-to-i32; subtract; convert-f32; multiply-scale`, and `ENCODED_NUMERIC_CODE_MIN`/`CODE_MAX` bound the code domain at `[0, 15]` for U4 and `[0, 255]` for U8.

**Inference — exhaustive over the finite code domain: if the scale is a normal F32, the decode is bit-identical under `FlushSubnormalsToZeroF32` and under a subnormal-preserving F32.** Take each step in order. The i32 subtraction of two values in `[0, 255]` yields an exact integer `v` with `|v| ≤ 255` and cannot overflow. Converting `v` to F32 is exact, because every integer of magnitude at most 255 is representable, so the converted operand is either `+0.0` or has magnitude at least `1.0` — it is never subnormal, whatever the conversion's rounding mode. The multiply's other operand is the scale. If `v = 0` the exact product is `+0.0` for a positive scale, which is a zero and not a subnormal, and which is what the registered exceptional contract `code-equals-zero-point-produces-positive-zero` already requires. If `v ≠ 0` then `|v · scale| ≥ scale`, so the exact product is below the F32 minimum normal `2^-126` only if the scale itself is. Therefore no operand and no result of the decode is subnormal when the scale is normal, and the flush has nothing to act on. This is exhaustive over 256 codes × 256 zero points rather than sampled.

**Measurement — the workload's own scales sit more than thirty orders of magnitude inside that condition.** Across all eight profiles and all 197 tensors the smallest scale is `1.358e-5` and the largest is `1.536e-1`; the F32 minimum normal is `2^-126 ≈ 1.1755e-38`, so the smallest measured scale exceeds the threshold by a factor of about `1.2e33`.

**Proposal — the correct repair is a stronger precondition, not a weaker contract.** The registered `QuantizeStrictAffine` already declares `positive_finite_scalar_predicate` on its scale operand with a typed invalid-input code. A *positive normal* predicate is strictly stronger: it admits nothing the current one rejects, it narrows the valid domain, and it is exactly what discharges the subnormal obligation above. With it, the strict-affine decode's numerical obligation becomes honourable on the measured Apple row without any authority substituting a realization — the outcome ADR 0076 requires and the one a weakened contract would have faked. **Inference.** This is why the profile below can name a Metal target at all, and it is filed as its own ticket rather than folded into the backend work, because it changes a semantic precondition and its runtime enforcement.

**Measurement — the decode's *integer* machinery is measured on the qualified row, exactly and no wider (closed 2026-07-31).** This paragraph previously carried the `Unknown` that nothing in this repository had measured integer arithmetic on any Apple GPU; E-1 below has since run, and [finding 32 of the numerical-behaviour record](../apple-targets/numerical-behaviour.md) with its retained record at `spikes/apple-targets/code-domain-integer-decode/results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv` holds the result: the complete decode chain — `u8` read, widen to `int`, subtract, `air.convert` to `float`, multiply — returns bit-exact agreement with the exact rational reference on every one of 1,310,720 normal-scale cells, both compilation paths agreeing cell by cell. The still-true halves stay: the retained numerical probe's dtype axis is exactly `f32`, `f16`, `bf16` (`numerical_probe.py:792`) — the sibling harness measured the decode, not a dtype widening — and the U4 extraction expression in `tiler-metal`'s emitter remains string-checked, absent from the compiled goldens, and never dispatched; a packed sub-byte extraction is a different measurement.

**Measurement — the absence check, with the control that makes it readable.** Over the 65 retained `.metal` probe sources, matching a declaration whose initializer contains an arithmetic operator returns **84** for `float`/`half`/`bfloat` and **0** for `int`/`long`/`short`/`char` and their unsigned forms, while a bare integer *declaration* matches 172 times — so integer types are present throughout as index and bit-pattern carriers and are never operands of arithmetic. Stating the control matters more than stating the zero: a search that had silently matched nothing would return the same zero, and the 84 and the 172 are what distinguish "the check ran and found none" from "the check did not run".

```sh
S=$(find spikes/apple-targets/results -name '*.metal')
echo "$S" | xargs grep -hoE '\b(float|half|bfloat) +[a-z_][a-z0-9_]* *= *[^;]*[-+*/][^;]*;' | grep -cv as_type
echo "$S" | xargs grep -hoE '\b(u?int|u?long|u?short|u?char)[0-9]? +[a-z_][a-z0-9_]* *= *[^;]*[-+*/][^;]*;' | grep -cv as_type
``` **Inference — the residual device risk is narrow but real and must not be argued away.** The derivation above shows the *values* are exact by construction, so what is unmeasured is not a numerical-behaviour question but a compile-and-dispatch one: whether the emitted MSL for a `u8` buffer read, an `int` subtraction, and an `int`-to-`float` conversion computes what the contract says on the qualified row. That is a smaller experiment than a numerical sweep, and it is filed as one below with its stop condition.

## The selected profile

**Proposal — per-output-channel strict-affine U8 to F32 over the workload's weighted projection operands.** Every field below is stated because the ticket requires code, scale, zero-point, grouping, axis, layout, and conversion identity for every surviving candidate, and because a profile that leaves one of them to a later reader is not a selection.

| Obligation | Value | Derivation |
| --- | --- | --- |
| Scheme family | `tiler::strict-affine@1` | the registered compound encoded-numeric scheme; no new family |
| Code type | `tiler::u8@1`, inclusive domain `[0, 255]` | derived from signedness class and logical width, not stored |
| Expressed type | `tiler::f32@1` | the workload is F32-widened throughout |
| Compute type | `tiler::f32@1` | the registered contract's `ENCODED_NUMERIC_COMPUTE_TYPE` |
| Codes component | role 1, `EncodedComponentShape::LogicalValue`, shape `[D_out, D_in]` | the weight's own shape |
| Scale component | role 2, `tiler::f32@1`, per-axis map over axis 0, shape `[D_out]` | axis 0 is the free index `o`; see the legality axis above |
| Zero-point component | role 3, `tiler::u8@1`, per-axis map over axis 0, shape `[D_out]` | the declared component type is the code type |
| Encode rounding | round-to-nearest ties-to-even | registered; ADR 0024 |
| Encode saturation | clamp to the inclusive code domain before integer conversion | registered |
| Exceptional inputs | NaN rejected as a semantic precondition | registered; ADR 0031 |
| Decode evaluation | widen code and zero point to i32, subtract, convert to f32, multiply by scale | registered; ADR 0032 |
| Observable materialization | preserve exact codes and associated parameters | registered |
| Physical storage | unpacked `StorageScalar::U8`, no packed encoding | eight bits is a whole storage unit |
| Scale value domain | positive **normal** finite | the honourability derivation above, strengthening the registered positive-finite predicate |
| Covered operands | the 196 weighted projection weights: 28 layers × `{q,k,v,o}_proj`, `{gate,up,down}_proj` | L2's contraction index structure 1 |
| Excluded from the first vertical | the tied `[151936, 1024]` embedding matrix | it is also a gather operand; see below |
| Target | `apple9-f32-unified-msl4-macos26`, the qualified row | L1's target qualification, selected indivisibly |

**Inference — why U8 and not U4, stated as an elimination rather than a preference.** Every U4 candidate fails the model observable in at least one variant, at every granularity and under every measured calibration; the best U4 profile that is *legal* under the governed contract, per-channel U4, agrees at 15 of 18 positions with the projections quantized and 6 of 18 with the embedding included. U4's advantage is bytes, and the control row above already shows that bytes alone decide nothing.

**Inference — why per-channel and not per-tensor.** Per-tensor U8 is the map Tiler already implements and it does not reproduce the C1 sequence in either variant. Per-channel U8 costs 0.3% more bytes than per-tensor U8 and reduces weight error by 3.8×.

**Inference — why per-channel and not per-block.** Legality, derived above, not accuracy: per-block's fused form consumes reassociation, which no contract registered for this workload grants.

**Inference — why unpacked U8 is a real advantage and not an incidental one.** [`prototype-quantized-value-vertical`](../../../tickets/prototype-quantized-value-vertical.md)'s cross-layer findings restricted the packed contract "to widths that divide eight" and recorded that "widths that cross bytes need a separately specified bitstream order and are unsupported until a real consumer supplies that contract"; its only implemented packed layout is `PackedU4LsbZeroTail`, and it had to carry a canonical-tail validation and a neighbour-clobbering-write refusal to make that layout safe. A U8 code needs no packed encoding, no bitstream order, no tail rule, and no partial-write ownership contract, so the selected profile removes an entire class of correctness obligation rather than inheriting it.

**Proposal — the tied embedding matrix is a measured extension, not part of the first vertical.** Including it moves the model's weight bytes from 1,064,714,240 (0.447 of F32) to 598,726,528 (0.251), with identical measured C1 behaviour — 17 of 18 greedy agreement and the exact sequence in both variants. That is the single largest memory term in the model and the case for it is measured. It is nonetheless excluded here because `tie_word_embeddings: true` means the matrix has two uses with different access relations: a contraction operand, which this profile covers, and a gather, which it does not. A per-row scale is the natural granularity for both — one scale per vocabulary token — but consuming a compound value through a gather is a second operation family, and [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](../../../tickets/admit-an-indirect-gather-family-for-tied-embedding-lookup.md) has not delivered it. The extension is filed below with that dependency rather than absorbed.

## How the contraction consumes it

**Fact — the semantic graph states a dequantization, and the physical plan is where it disappears.** The program is `input(compound weight) → DequantizeStrictAffine → Contraction(activation, weight)`. Nothing about packing, fusion, or operand access enters the semantic graph: the architectural contract requires that physical choices not densify into it, and a "packed contraction operation" would be exactly that densification.

**Inference — an explicit dequantization boundary that materializes F32 defeats the entire purpose, and the arithmetic says so in two branches.** L3 measured the complete decode vocabulary projection reading the whole `[151936, 1024]` F32 weight — 622,329,856 bytes — per dispatch at 4,247 µs, or 146 GB/s, and concluded "the cell is bandwidth-bound … and no arithmetic schedule can move it". Under the selected profile that operand is 156,342,144 bytes, 0.2512 of the F32 form. Now take the two ways to spend it. *Materialize per dispatch:* read 156 MB of codes and parameters, write 622 MB of F32, then read 622 MB back — 1.401 GB of traffic against the F32 baseline's 0.622 GB, so a quantized decode would be 2.25× *slower* than the baseline it is meant to improve. *Materialize once and keep the result resident:* the 622 MB of F32 weights are live again and the memory win is exactly zero. **Inference.** Only fusing the decode into the contraction's operand access delivers both, and that is the physical route this profile selects.

**Inference — the fused form is bit-identical to the materialized form for this profile, and that is a derivation rather than a hope.** The concern the architectural contract names is a deleted materialization rounding point: eliding a store can change results when the elided value would have been rounded. Here the decode's compute type and its expressed type are both `f32`, so the value written by a materializing plan and the value held in a register by a fused plan are the same F32 bits, and eliding the store rounds nothing away. **Inference — and this is a property of *this* profile, not of fusion.** A profile whose compute type were wider than its expressed type would delete a real rounding point, and the same fusion would then be a semantic change requiring its own permission.

**Inference — the fused decode consumes no numerical permission, which is what keeps L3's survivor intact.** L3's `tiled` realization is "the surviving strict realization", attributed uniquely to `strict_fold+ftz` and bit-identical to the host oracle at every profile cell. Fusing a per-channel decode changes the *value* of each contributor and changes neither the contributor sequence, the ascending-`d` order, nor the unseeded first-product start, so the fused kernel is still a strict fold. Reassociation, permutation, and distributivity all stay unconsumed — including the tempting integer-domain factorization `s[o] · (Σ a·w − z[o]·Σ a)`, which is a distributivity rewrite and which L3 records as "absent; no contract Tiler can express grants the third".

**Fact — the emission constraint L3 measured applies unchanged.** `-ffp-contract=off` governs statements the emitter writes separately and has nothing to say about an instruction whose fusion *is* the instruction; L3 demonstrated this by recompiling with `-ffp-contract=fast` and watching `direct`, `tiled`, and `ksplit_contiguous` flip to the fused value. The fused decode adds a multiply and a subtract to the contraction's inner statement, so the per-statement emission rule is the thing holding the line and a matrix-instruction lowering remains refused for the same reason it already is.

## The normative reference, the accumulation, and the error criteria

**Proposal — the normative reference is Tiler's own reference evaluator applied to the quantized program, with zero tolerance.** [`prototype-quantized-value-vertical`](../../../tickets/prototype-quantized-value-vertical.md) already established the shape: "The implemented strict-affine comparator has zero tolerance because its contract specifies exact `f32` evaluation." The decode's evaluation order is fully determined and its only rounding is one correctly-rounded F32 multiply, so a backend that disagrees with the reference by one bit is wrong rather than approximate. **Inference — that is not the same as saying the decode cannot produce an exceptional value.** The vertical's own final audit found that finite valid inputs can still overflow to infinity, and the derivation above bounds the small end rather than the large one: `|v| ≤ 255` and a scale near `f32::MAX` overflow. No measured scale comes close — the largest anywhere in the checkpoint is `1.536e-1` — but the reference and the backend must agree on infinity, not be assumed never to reach it.

**Inference — the model-level comparison against the F32 model is a different question and must not be folded into that one.** L1 measured the pinned reference's own F32 reordering envelope at a whole-vocabulary maximum of `2.048e-4` and a top-32 maximum of `7.82e-5`. The gentlest surviving candidate here has a *median* whole-vocabulary deviation of `1.08e-1` and a top-32 maximum of `3.00`. A quantized program is a different computation, not a different realization of the same one, so the F32 bounded-error comparison cannot be reused for it, and a compiler-correctness tolerance derived from it would be either vacuous or unachievable. The compiler's obligation is exactness against the quantized program's own reference; whether the quantized program is an acceptable approximation of the F32 model is an ingestion and qualification question that [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) owns, and this record supplies its measured inputs rather than a budget.

| Obligation | Value for this profile | Derivation |
| --- | --- | --- |
| Reference | Tiler's reference evaluator on the same quantized program | the vertical's exact comparator |
| Comparison | exact bits, zero tolerance | exact F32 evaluation over a finite integer domain |
| Accumulator dtype | `tiler::f32@1` | unchanged from L3; its D-6 stays open and is not reopened here |
| Result dtype | `tiler::f32@1` | the contraction's declared result |
| Contributor sequence | ascending `d`, `0 .. K-1` | unchanged by the fused decode |
| Seed | none; the accumulator starts at the first product | L3's measured `negative_zero_seed` counterexample applies unchanged |
| Order permissions | reassociation Forbidden, permutation Forbidden, distributivity absent | the governed contract; the legality axis above |
| Subnormals | `FlushSubnormalsToZeroF32`, discharged for the decode by the normal-scale precondition | the honourability derivation above |
| NaN results | `tiler::canonical-arithmetic-nan-f32@1` | inherited from the contraction's signature; L3's D-8 stays open |
| Determinism | plan deterministic | nothing here uses an atomic |

**Proposal — artifact identity.** The vertical's identity suite already perturbs scheme, static contract fields, component roles, order, types and maps, embedded scale bits, and storage encoding one at a time, and already separates a runtime scale payload (changes evaluation, not identity) from an embedded constant (changes identity). The selected profile adds exactly one identity-affecting field: the parameter map's *axis and extent*. Two artifacts differing only in which axis a per-axis scale indexes must have different identities, and that perturbation belongs with the map's implementation.

**Proposal — weight validation, as runtime preconditions rather than trust.** Before routing commit: every scale positive, finite, and **normal**; every zero point within `[0, 255]`; the scale and zero-point tensors' extent equal to the codes' axis-0 extent; and the component bindings resolved by role rather than by slot position. **Inference — one obligation the U4 profile carries does not arise here**: there is no packed tail, so there is no canonical-tail bit check and no partial-write ownership contract. [`implement-first-runtime-semantic-value-precondition-enforcement`](../../../tickets/implement-first-runtime-semantic-value-precondition-enforcement.md) owns the enforcement and is a structural dependency of the backend work below.

**Inference — the selected profile materializes no compound value internally, and the graph does not yet reflect that.** The weights arrive as role-addressed compound *interface inputs*; the executed program contains no `Quantize` and no `Assemble`, so the only compound value is an input, which the vertical already proved end to end. [`group-internal-compound-materializations-by-logical-value`](../../../tickets/group-internal-compound-materializations-by-logical-value.md) is therefore not a *direct* dependency of the backend work, and no edge was added for it.

**Fact — it nonetheless remains a transitive prerequisite, through a chain worth naming rather than leaving to a reader to discover.** `implement-first-quantized-backend-profile` → `implement-first-runtime-semantic-value-precondition-enforcement` → `carry-semantic-enforcement-plans-through-program-and-artifact` → `admit-strict-affine-quantize-physical-candidate` → `group-internal-compound-materializations-by-logical-value`. The chain exists because the runtime-enforcement vertical is scoped to strict-affine **`Quantize`** preconditions, and a `Quantize` *does* produce a compound value internally.

**Inference — that is a scope mismatch in an existing ticket, not a requirement of this profile, and it is recorded rather than routed around.** What the selected profile needs enforced is the value domain of an *input* compound value — positive normal scale, in-range zero point, parameter extents agreeing with the codes' axis-0 extent — reached through `Dequantize` and binding conformance, not through `Quantize`. Depending on the `Quantize` vertical therefore drags in a physical route and an internal-grouping capability the profile never exercises. Resolving it belongs to the enforcement ticket's owner — by widening that vertical's first subject to a `Dequantize` input, or by a sibling — and a dated note on it says so. This record does not file a competing ticket for an authority that already has one.

## Memory and performance against the F32 baseline

**Measurement — the weight budget, against L1's arithmetic.** The measured F32 total, 2,384,199,680 bytes, reproduces L1's figure exactly from the checkpoint's own header, which is the cross-check that the two records are describing the same model.

| Configuration | Weight bytes | vs F32 |
| --- | --- | --- |
| F32 (L1's baseline) | 2,384,199,680 | 1.000 |
| BF16 storage control | 1,192,230,912 | 0.500 |
| Selected profile, projections only | 1,064,714,240 | 0.447 |
| Selected profile plus the tied embedding | 598,726,528 | 0.251 |

**Inference — quantizing the weights makes the KV cache the dominant term, and L5 should know that before it designs one.** Composing with L1's KV-cache table at the longest benchmark row, B1-d at 8,320 context:

| Configuration | Weights | KV cache (F32) | Total | vs F32 total |
| --- | --- | --- | --- | --- |
| F32 | 2,384,199,680 | 1,908,408,320 | 3.998 GiB | 1.000 |
| Selected, projections only | 1,064,714,240 | 1,908,408,320 | 2.769 GiB | 0.693 |
| Selected plus embedding | 598,726,528 | 1,908,408,320 | 2.335 GiB | 0.584 |

At the F32 baseline the weights are 56% of resident state; with the embedding quantized they are 24% and the KV cache is 76%. **Inference.** Quantizing weights alone therefore has a floor on what it can do for the long rows, and the next memory question for this workload is the cache rather than the weights — which is [`design-autoregressive-state-and-kv-cache`](../../../tickets/design-autoregressive-state-and-kv-cache.md)'s, not this record's.

**Inference — an analytical performance projection, and it is not a measurement.** L3's `t_vocab_full` cell is bandwidth-bound at 146 GB/s for its best candidate; under the selected profile its operand is 0.2512 of the F32 bytes, so *if* the cell remains bandwidth-bound at the same achieved rate a fused quantized decode would land near 1,067 µs against the measured 4,247 µs. Every part of that sentence is a hypothesis: the fused kernel does more arithmetic per byte, the achieved bandwidth of a `u8` read stream is unmeasured on this host, and nothing has dispatched an integer instruction on an Apple GPU in this repository. **This record makes no device-optimal claim and none may be made from it.** [`calibrate-device-cost-models`](../../../tickets/calibrate-device-cost-models.md) and the device measurement filed below are structural prerequisites of any such claim, and the cost model must keep the packed-fused, explicit-dequantize, and F32 candidates as separately costed alternatives rather than assuming the first wins.

**Inference — the prefill half is a different question and the answer may be no.** L3 measured prefill as compute-bound rather than bandwidth-bound — `tiled` at 1,602 µs for a 128×3072×1024 cell — so the weight-byte reduction buys much less there, and the fused decode's extra arithmetic per weight element is a cost with no bandwidth saving to pay for it. A profile that helps decode and hurts prefill is a plausible outcome and the cost model must be allowed to select per occurrence, which is what "optimal means the lowest-cost valid plan, not the largest fused kernel" already requires.

## Where this lands in the dtype maturity ledger

**Fact — this record moves no maturity cell, and that is the correct outcome rather than an omission.** [The ledger](../../dtype-support.md) classifies what Tiler has *built*; a research selection builds nothing, and its own graph policy says "Advance only the cells the selected profile actually implements or tests". Recording the selection in a cell would be exactly the promotion of recognition into support that the ledger exists to prevent.

**Fact — what the selection does change is two triggers, which have now fired.** The `Strict-affine U8/F32` row's trigger required "a selected workload must name the U8 physical representation, component ABI, candidate, cost, target, runtime predicates, and conformance corpus independently of U4"; the `Other affine and OCP MX schemes` row's trigger required a profile naming "its exact code/expressed/scale/compute types, parameter map, grouping, storage, ABI, runtime predicates, target, cost, and corpus". This record names both sets. The ledger's evidence prose is updated to point here; the cells are not.

**Proposal — the classification each delivery ticket below may claim, kept apart because the ticket requires all five to be separately stated.** A selected code width, a packed layout, and a native instruction cannot stand in for one another.

| Layer | Claim after this record | Claim the delivery graph would earn |
| --- | --- | --- |
| Logical scalar type (`tiler::u8@1`) | tested guarantee (identity and code domain) | unchanged; integer *arithmetic* is a separate vertical this profile does not trigger |
| Numerical interpretation (per-channel strict-affine U8/F32) | absent — the scheme validator admits only the two per-tensor contracts | tested guarantee, from the parameter-map ticket |
| Parameter map (per-axis over axis 0) | architectural seam — `ParameterIndexMap` exists, only `per_tensor()` is constructible | implemented mechanism, then tested guarantee |
| Physical storage carrier and encoding (unpacked `StorageScalar::U8`) | implemented mechanism as a *carrier*; no U8 compound value uses it | tested guarantee, from the physical-vocabulary ticket |
| Kernel access and arithmetic type (`KernelType::U8`, `I32`, `F32`) | implemented mechanism, exercised only by the U4 path | tested guarantee at a per-axis parameter access |
| Target-family dispatchability | absent; no dtype dispatchability axis exists, and the axis ticket was superseded. Finding 32 measures one decode chain's arithmetic on the qualified row and is deliberately *not* a dispatchability row — an observed kernel is not an admission authority | a measured `(Apple9 macOS, strict-affine U8/F32)` row through the caller-declared-profile boundary, or `Unknown` |
| Backend execution | absent; nothing quantized has ever executed on a device | gated on the device measurement below |

## Bounded experiments this record could not run

Each states inputs, outputs, and a stop condition, because an unmeasured claim recorded as a gap is evidence and an unmeasured claim recorded as an assumption is not.

**E-1 — code-domain integer arithmetic on the qualified Apple row.** *Inputs:* kernels on the `apple9-f32-unified-msl4-macos26` row that read a `u8` buffer, widen to `int`, subtract an `int` zero point, convert to `float`, and multiply by a `float` scale, compiled under the governed flags, over the complete 256×256 code/zero-point grid and a scale corpus spanning the measured range plus the normal/subnormal boundary. *Outputs:* the returned bits against the exact rational evaluation, and the delivered subnormal behaviour at a deliberately subnormal scale. *Stop condition:* either every cell matches the reference and the subnormal case is observed flushing exactly where the derivation says it would, or a divergence is found and named. *Run 2026-07-31 by [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](../../../tickets/measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md), landing on the stop condition's first branch:* all 28 cases and 1,835,008 dispatched cells agree — 1,310,720 normal-scale cells bit-identical to the exact reference, all 14 offline/runtime comparisons `agree`, `code == zero_point` returning `+0.0` in 256/256 diagonal cells of every case — and the subnormal-scale flush acts on the *operand* (64,770 exactly-normal products still returned signed zeros at scale `2^-127`), which is precisely where the derivation put it; at `2^-126` nothing flushes. Retained record: `spikes/apple-targets/code-domain-integer-decode/results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/`. The boundary is finding 32's: one family, one GPU, one toolchain and flag row, `u8` codes, one non-overflowing subtraction, no packed extraction, no timing — E-2 is untouched and still blocks any device-optimal claim.

**E-2 — the fused decode's achieved bandwidth against the F32 baseline.** *Inputs:* L3's `t_vocab_full` and `w_decode_kv` cells with the weight operand replaced by codes plus per-channel parameters, under L3's own timing procedure — one process, five interleaved A/B rounds of seven timed dispatches, settled minimum over rounds 1–4. *Outputs:* settled minimum GPU time and achieved bytes per second, against the retained F32 rows. *Stop condition:* the quantized cell's achieved rate is within the measured spread of the F32 cell's, or the difference is attributed. *Why it is not run here:* the probe hard-codes `float` in all three of its producers — buffers, `sizeof`, PRNG, digest, readback, the MPS descriptors, and an exact-arithmetic binary32 oracle with no integer analogue. *Blocks:* any device-optimal claim.

**E-3 — a second accuracy row.** *Inputs:* the selected profile against a B1 prompt length and at least one prompt outside C1. *Outputs:* greedy agreement and logit deviation on the same form as the retained record. *Stop condition:* agreement holds, or a length or prompt where it fails is named. *Why it is not run here:* L1 deliberately made C1 the only fully retainable row, and a B1-length accuracy comparison needs the harness and the retention policy that [`design-model-level-qualification-and-optimization`](../../../tickets/design-model-level-qualification-and-optimization.md) owns. *Blocks:* generalizing the accuracy reading past C1 — which this record does not do.

**E-4 — calibration beyond min-max and quantile clipping.** *Inputs:* an MSE-optimal or error-compensating calibration against the same profile set. *Outputs:* the same relative-error table. *Stop condition:* the granularity ordering is preserved or a reordering is found. *Why it matters and why it is bounded:* a better calibration improves every candidate and the measured sweep bounds how much it can *reorder* them, not how much it can improve them, so a future calibration could revisit the U4 rows on accuracy — it could not revisit per-block on legality.

## Delivery tickets

Dependency-ordered. Each is scoped to surviving work only; nothing is filed for U4, for per-block, for MX, or for FP8 as a realization.

| Order | Ticket | Outcome |
| --- | --- | --- |
| 1 | [`admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode`](../../../tickets/admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode.md) | The strict-affine scale predicate is strengthened from positive-finite to positive-normal, the subnormal obligation the decode declares becomes dischargeable, and the Metal refusal that currently fires is shown to fire for a subnormal scale and not for a normal one. |
| 2 | [`implement-workload-selected-quantized-parameter-maps`](../../../tickets/implement-workload-selected-quantized-parameter-maps.md) *(refined)* | The per-axis map over axis 0 becomes the first non-per-tensor `ParameterIndexMap`, with rank-1 scale and zero-point components, reference evaluation, transform preservation, and typed refusal of every unselected map. |
| 3 | [`widen-the-physical-vocabulary-for-per-axis-quantized-component-access`](../../../tickets/widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md) | A compound value's parameter component is addressed by a projection of the iteration domain rather than as a rank-zero scalar, through signature verification, KIR identity, ABI compatibility, lowering and emission, with negative unsupported-combination tests. |
| 4 | [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](../../../tickets/measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) | **Done 2026-07-31.** E-1 ran and landed on agreement: every cell matched and the subnormal flush is operand-side, exactly as derived. Finding 32 and the retained record above hold the result and its boundary. |
| 5 | [`fuse-quantized-weight-decode-into-the-strict-contraction`](../../../tickets/fuse-quantized-weight-decode-into-the-strict-contraction.md) | The decode becomes an operand access of L3's surviving `tiled` realization, consuming no numerical permission, bit-identical to the materializing form, with the materializing alternative retained and separately costed. |
| 6 | [`implement-first-quantized-backend-profile`](../../../tickets/implement-first-quantized-backend-profile.md) *(activated and refined)* | The selected profile compiles, executes, and matches the reference exactly on the qualified row, with every unselected scheme, width, map, encoding, and target refused by name. |
| 7 | [`extend-the-selected-quantized-profile-to-the-tied-embedding-matrix`](../../../tickets/extend-the-selected-quantized-profile-to-the-tied-embedding-matrix.md) | The largest memory term joins the profile once a gather can consume a compound value. |

**Fact — what is deliberately *not* filed.** No ticket for weight ingestion: converting checkpoint bytes into the selected profile's components is the same question L1 left open for the BF16-to-F32 conversion — "whether it is a Tiler semantic operation or a host-side ingestion step is L6's question" — and [`design-model-ingestion-and-complete-execution`](../../../tickets/design-model-ingestion-and-complete-execution.md) owns it. No ticket for internal compound grouping, for the reason given above. No competing ticket for the model-level accuracy budget, which L8 owns. No ticket for a packed sub-byte contraction, because no sub-byte candidate survived.

## What this record does not decide

- **Whether a per-block scheme is a good idea.** It decides only that no contract registered for this workload admits its fused form. A caller granting reassociation reopens it at a better measured accuracy and a comparable byte cost.
- **The accumulation dtype.** L3's D-6 is unchanged; a wider accumulator remains a live option owned by [`implement-parallel-reduction-strategies`](../../../tickets/implement-parallel-reduction-strategies.md).
- **Whether the per-combine canonical-NaN obligation survives a fused decode.** L3's D-8 asked it of a matrix instruction; the fused decode is scalar and can interpose, so the question is unchanged rather than answered.
- **Any accuracy claim beyond the C1 row.** Eighteen positions, one prompt, one checkpoint.
- **Any device claim at all.** Nothing quantized has executed on any GPU in this repository, and the analytical projection above is a hypothesis with a named experiment attached.
- **Activation quantization, mixed precision, and KV-cache quantization.** None is measured, none is filed, and the last of the three is where the memory question goes next.
