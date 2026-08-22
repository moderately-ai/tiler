---
id: scope-the-padding-and-cropping-family
title: Scope the padding and cropping family
status: deferred
priority: p3
dependencies: []
related: [scope-the-windowed-reduction-and-convolution-family, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, structural, numerics, deferred]
---
## User-visible outcome

A padded or cropped tensor is one governed operation whose pad value is a stated numerical participant, so that a pass eliding the materialization owes a neutrality proof instead of assuming zero is neutral.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-25 is atomic, with interior, edge, and negative padding as attribute values of one family, "low, high, and interior pad amounts per axis, signed so that a negative amount crops", and one D5 sentence that is the whole reason this is its own track: "The pad value participates in downstream numerics and is **not** neutral by virtue of being an identity element."

**Fact — the counterexample is already in the corpus and is exact.** [Numerical semantics](../docs/numerical-semantics.md) keeps empty result, algebraic identity, and safe physical padding as three separate facts: a strict floating sum may return `+0.0` for an empty domain, yet adding `+0.0` to a singleton `-0.0` under round-to-nearest produces `+0.0`, so `+0.0` is not bitwise-neutral padding for that reduction even though it is its empty result. The same obligation is what a tiled contraction owes for a ragged contracted extent; [roadmap](../docs/roadmap.md) Milestone 6 states that under **Fact — K-padding is not free, and the contract already says so.**

**Inference — this is why padding is not grouped with the other structural families.** F-22, F-23, F-24, F-26, and F-27 have no numerical content at all; padding introduces a value into the result that downstream arithmetic reads. Grouping it with them would give one track two numerical contracts, which is exactly the split this record's partition rule requires.

**Fact — reflect, edge, and wrap modes are out, and the reason is an access class rather than a preference.** The taxonomy records them as "separate coordinate maps and are unsupported until the piecewise map class exists" — [Q-SHAPE-006](../docs/open-questions.md#q-shape-006--finite-piecewise-access-maps), whose trigger is unfired.

## Activation trigger

A named workload needs explicit padding or cropping — most plausibly a convolution or pooling occurrence, whose family is [`scope-the-windowed-reduction-and-convolution-family`](scope-the-windowed-reduction-and-convolution-family.md)'s and which carries the same pad-value obligation inside its own window attributes. A *physical* tile pad does not fire it: that is a schedule choice owing a neutrality proof, not a semantic operation.

## What the work would be, when it starts

The per-axis signed low/high/interior attributes with the cropping sign convention stated; the constant-pad form's rank-zero pad value operand of the same type; the refusal of reflect, edge, and wrap until the piecewise class exists; the materializing oracle; the guarded-read lowering; and — the part that is not bookkeeping — the neutrality obligation an eliding pass must discharge, written as a proof requirement under the selected numerical contract rather than as a note.

## Explicit non-goals

- The physical tile pad a tiled schedule performs, which is a schedule obligation under the same neutrality rule and not this family.
- Reflect, edge, and wrap modes, which need the piecewise access class.
- The windowed family's own padding attributes, which belong to that family's signature even though they carry this obligation.

## Closes when

The family has a signed per-axis attribute schema, a materializing oracle, a guarded-read lowering, and a written neutrality obligation for elision — with the obligation exercised by at least one case where a plausible pad value is not neutral for the consuming reduction.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-24** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-25 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named workload needs explicit padding or cropping; the pinned workload's only padding-shaped concern is the additive causal mask, which is a bound `f32` program input rather than a pad. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** No convolution, pooling, or explicit pad/crop workload is selected. The causal mask remains ordinary bound tensor data, and physical tile-boundary questions remain schedule obligations rather than a semantic padding occurrence.
- 2026-08-10 — **not fired.** No named workload needs explicit padding or cropping; no pad/crop OpKey is registered under semantic construction sites; the causal mask remains bound F32 program input rather than a pad op; physical tile pad remains schedule. The 2026-08-05 "46 governed keys / eighteen registered operation keys" census is a dated snapshot, not a live count.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it ~~returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output.~~ **Correction — 2026-08-22 (the total is withdrawn, not re-corrected).** The struck wording is kept as the claim it retired, not as a live instruction. Three things are wrong with it. Its `[a-z0-9-]+` class silently drops the five underscore-bearing MX keys — `tiler::mxfp4_e2m1@1`, `tiler::mxfp6_e2m3@1`, `tiler::mxfp6_e3m2@1`, `tiler::mxfp8_e4m3@1`, and `tiler::mxfp8_e5m2@1` — so the corrected class `[A-Za-z0-9_-]+` returns **55** here rather than 50. The figure is scope-bound rather than absolute: that same corrected pattern returns 63 under `crates/` and 72 across the repository outside `tickets/`, so the directory argument decides the answer as much as the tree does. And it was never a key population at all — governed keys are constructed, not spelled, by helpers such as `governed_op("assemble-strict-affine")` in `crates/tiler-ir/src/semantic/quantization.rs`, so `tiler::assemble-strict-affine@1` and `tiler::quantize-strict-affine@1` are governed keys carrying no bare literal and are absent from all 55. A total that has drifted 46 to 47 to 50, that moves whenever any unrelated dtype is admitted, and that has never once moved for the event this trigger watches is not this trigger's change-detector; the key-named check below replaces it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the padding and cropping family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(pad|crop)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::pad-f32@1` to that census makes the same command print `tiler::pad-f32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
