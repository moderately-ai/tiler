---
id: scope-the-windowed-reduction-and-convolution-family
title: Scope the windowed reduction, pooling, and convolution family
status: deferred
priority: p3
dependencies: []
related: [scope-the-padding-and-cropping-family, decide-whether-a-contraction-may-consume-more-than-two-operands, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, reductions, convolution, deferred]
---
## User-visible outcome

`RQ-OP-09` is answered: a windowed reduction is either an atomic family, a window structure over the contraction's index structure, or a region-bearing reduction — and the answer says whether [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md)'s index structure is the corpus's general mechanism or the contraction's own.

## Why this is deferred rather than open

**Fact — the question and its test.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-09` "Closes by attempting to express one strided, dilated, padded convolution as an index structure under ADR 0087's five structural rules. If the padding predicate cannot be carried without a piecewise map ([Q-SHAPE-006](../docs/open-questions.md#q-shape-006--finite-piecewise-access-maps)), the index-structure candidate is refuted and the family is atomic."

**Fact — the family inherits two obligations and adds one.** F-33 takes input, filter, accumulator, and result types as separate fields — "the accumulator type is where quantized convolution's meaning lives" — inherits every reduction obligation from F-28, and adds the window: extents, strides, dilation, padding, and a layout naming which axis is batch, channel, and spatial. Its padded window's pad value "is a numerical participant exactly as in F-25".

**Fact — the alternatives are not faster realizations of one contract.** "Winograd changes the arithmetic and is therefore a different numerical contract, not a faster realization of the same one." A reader who treats im2col and Winograd as schedule choices for one family has already lost the numerical contract.

**Inference — the deferral is honest because the answer schedules nothing today.** Nothing widens the contraction's index structure and no workload names a convolution, so a refutation and a confirmation both produce a record and no work. Running the test early would still be cheap, which is why the trigger below admits a second route that is not a workload.

## Activation trigger

Either a named workload requires a windowed reduction, pooling, or convolution occurrence, **or** a proposal is made to widen the standard contraction key's index structure beyond binary contraction — because the second is exactly the case where knowing whether a window fits inside that structure stops being academic.

**Correction — 2026-08-19 (key retired; the trigger is unchanged in substance).** This trigger named `tiler::strict-tensor-contraction-f32@1` as the current standard contraction key. That key is **retired from the standard vertical** under [ADR 0112](../docs/decisions/0112-replace-the-strict-contraction-key-with-a-permission-indexed-successor.md), which replaced it with `tiler::tensor-contraction-f32@1` (`crates/tiler-ir/src/semantic/contraction.rs`, anchor `is the documented successor to the`); `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs` pins that the old key can no longer produce a program. The trigger is stated against the *standard contraction key* rather than a spelling, because what makes it fire is a proposal to widen that key's ADR 0087 index structure beyond binary contraction — a property the successor inherits unchanged. The successor is `tiler::tensor-contraction-f32@1`; naming it here rather than the retired spelling is the whole repair, and the ADR 0087 test in the work section below is unaffected.

**Command recheck — 2026-08-19.** The 2026-08-05 log entry's recheck command still runs but **no longer returns the count it records**: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` returns **50** governed keys at this base, not the 46 stated there. The entry's *conclusion* is unaffected and the trigger stays unfired — the windowed/convolution family's key is still absent from that list, which is what the check exists to establish. The count is dated evidence about 2026-08-05 and is left in that entry rather than rewritten; this note is the current reading.

## What the work would be, when it starts

Run the test: express one strided, dilated, padded convolution as an ADR 0087 index structure and report which of the five structural rules it meets and which it cannot. Then, whichever way it goes, state the window attributes, the layout naming, the accumulator field, the pad value's numerical participation, the direct nested-loop oracle, and the explicit statement that im2col and Winograd are separate numerical contracts rather than realizations.

## Explicit non-goals

- Widening the contraction key. If the test succeeds it is a finding about the structure's generality, and the widening is separate work with its own acceptance boundary.
- Any Winograd or im2col realization, which need their own contracts before they need a schedule.
- Padding as a general family, which is [`scope-the-padding-and-cropping-family`](scope-the-padding-and-cropping-family.md)'s even though this family carries the same obligation inside its own attributes.

## Closes when

`RQ-OP-09` is answered against the worked convolution, with the failing structural rule named if it is refuted and the carried structure shown if it is not — and the taxonomy's `RQ-OP-09` row names the answer.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-28** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-33 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired, on both routes.** No workload names a convolution, pooling, or windowed reduction occurrence, and no proposal widens the contraction's index structure — the one live multi-operand question is [`decide-whether-a-contraction-may-consume-more-than-two-operands`](decide-whether-a-contraction-may-consume-more-than-two-operands.md), which is about operand count rather than window structure. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired, on both routes.** No selected workload names convolution, pooling, or a windowed reduction, and the live contraction-boundary work still concerns declared-input/operand cardinality rather than adding window structure to ADR 0087's family.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it ~~returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output.~~ **Correction — 2026-08-22 (the total is withdrawn, not re-corrected).** The struck wording is kept as the claim it retired, not as a live instruction. Three things are wrong with it. Its `[a-z0-9-]+` class silently drops the five underscore-bearing MX keys — `tiler::mxfp4_e2m1@1`, `tiler::mxfp6_e2m3@1`, `tiler::mxfp6_e3m2@1`, `tiler::mxfp8_e4m3@1`, and `tiler::mxfp8_e5m2@1` — so the corrected class `[A-Za-z0-9_-]+` returns **55** here rather than 50. The figure is scope-bound rather than absolute: that same corrected pattern returns 63 under `crates/` and 72 across the repository outside `tickets/`, so the directory argument decides the answer as much as the tree does. And it was never a key population at all — governed keys are constructed, not spelled, by helpers such as `governed_op("assemble-strict-affine")` in `crates/tiler-ir/src/semantic/quantization.rs`, so `tiler::assemble-strict-affine@1` and `tiler::quantize-strict-affine@1` are governed keys carrying no bare literal and are absent from all 55. A total that has drifted 46 to 47 to 50, that moves whenever any unrelated dtype is admitted, and that has never once moved for the event this trigger watches is not this trigger's change-detector; the key-named check below replaces it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the windowed reduction, pooling, and convolution family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(convolution|conv|pool|window)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::convolution-f32@1` to that census makes the same command print `tiler::convolution-f32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
