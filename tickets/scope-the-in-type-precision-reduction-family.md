---
id: scope-the-in-type-precision-reduction-family
title: Scope the in-type precision reduction family
status: deferred
priority: p3
dependencies: []
related: [test-the-directional-conversion-pair-generalization, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, precision, deferred]
---
## User-visible outcome

A caller who needs a value rounded to a narrower exponent and significand *and left in its own type* can say so, and the operation is not modelled as a dtype conversion, which it is not.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-21 is atomic, one operand and one result, where "operand and result carry the *same* resolved type; the reduction is to a narrower exponent and significand and back", carrying exponent-bit and mantissa-bit attributes, with "round-to-nearest-ties-to-even to the reduced significand, then overflow to a signed infinity or underflow to a signed zero if the reduced exponent range cannot hold the result". Its D8 is a single sentence and it is the reason this is a separate track: "It is not a dtype conversion and must not be modelled as one, because the result type never changes."

**Fact — the grouping that survives three primary sources puts it alone.** The taxonomy records that TOSA keeps `CAST` and `RESCALE` in one category, ONNX separates five conversion operations, and StableHLO separates `convert`, `uniform_quantize`, `uniform_dequantize`, and `reduce_precision`; the grouping that survives all three is "by *what the transition preserves*: F-18 and F-19 change the type, F-20 changes the numeric interpretation while the code type may be unchanged, and F-21 changes neither. Those are three different obligations, and collapsing any two of them puts an unstated rounding into a family whose signature does not mention rounding."

**Fact — the matrix has no row for it.** F-21 is one of the twenty-five families the join table lists under *(no matrix row today)*, and the cast-and-convert row's name does not reach it. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 47 governed keys today, comprising the dtype identities, the ULP metric key, and the nineteen registered operation keys (including `tiler::gather-f32@1`); the family's key is absent from that list.

**Inference — its physical route already exists and that is not a reason to admit it.** [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) places F-21 in the *covered — direct scalar or map route* class, so the profile would owe it a scalar kernel the day it is admitted. What is missing is a producer: nothing in the corpus asks for a deliberately reduced-precision value in its own type.

## Activation trigger

A named producer requires precision reduction within one type — a mixed-precision emulation, a deliberate accuracy-degradation study, or a frontend lowering that spells `reduce_precision` — and can state the exponent and mantissa bits as identity rather than as configuration.

## What the work would be, when it starts

The key and its two attributes; whether the exponent and mantissa bit counts are part of identity, which they must be by the same argument that makes the normalization's `eps` part of identity — two reductions differing only in mantissa bits are different operations; the exact round-trip oracle; the overflow-to-signed-infinity and underflow-to-signed-zero rules stated separately from the rounding rule; and the scalar emission, with the note that a native instruction is an optimization rather than the definition.

## Explicit non-goals

- Any conversion family. If the result type changes it is F-18 or F-19 and belongs to [`test-the-directional-conversion-pair-generalization`](test-the-directional-conversion-pair-generalization.md).
- Quantization. F-20 changes the numeric interpretation; this changes neither type nor interpretation.
- A target-driven emulation of a reduced-precision dtype, which is an execution-only format question on the dtype axis.

## Closes when

The family has a key whose attributes are part of identity, an exact round-trip oracle, separately stated overflow and underflow rules, and a scalar emission — or is recorded as permanently unneeded with the producer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-23** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-21 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named producer requires precision reduction within one type; the corpus's only in-type transformation is `ConvertOp::CanonicalizeF32Nan`, an `f32`-to-`f32` NaN canonicalization, which changes no exponent or significand width. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** No producer asks for an in-type exponent/significand reduction. BF16 widening and exact BF16 constant reinterpretation change type or physical spelling respectively; neither is a same-resolved-type `reduce_precision` operation.
- 2026-08-10 — **not fired.** No named producer requires in-type exponent/significand reduction; no semantic `reduce_precision` OpKey. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 47 governed keys, comprising the dtype identities, the ULP metric key, and the nineteen registered operation keys (including `tiler::gather-f32@1`); the family's key is absent. Same-type kernel transforms present today are `ConvertOp::CanonicalizeF32Nan` and `ConvertOp::CanonicalizeBf16Nan` — both width-preserving NaN canonicalizations, neither F-21. The 2026-08-05 log's "only … CanonicalizeF32Nan" wording and "46 / eighteen" census, and the matrix Fact's pre-correction "twenty-three" / "46 / eighteen", are superseded by this entry and the corrected Fact above.
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the in-type precision reduction family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(precision-reduce|reduce-precision|round-to-precision)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::reduce-precision-f32@1` to that census makes the same command print `tiler::reduce-precision-f32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
