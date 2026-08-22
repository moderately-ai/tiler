---
id: scope-the-geometric-resampling-family
title: Scope the geometric resampling family
status: deferred
priority: p3
dependencies: [admit-an-indirect-gather-family-for-tied-embedding-lookup]
related: [scope-the-padding-and-cropping-family, derive-the-operation-family-and-signature-delivery-graph, revise-adr-0108-with-a-complete-data-dependent-index-vertical, emit-the-indirect-gather-on-metal]
scopes: [research/semantic-graph, research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, resampling, deferred]
---
## User-visible outcome

A resize, a grid sample, or an affine warp is one family carrying all four of the attributes that decide its result, so that two callers asking for "bilinear" cannot get two different tensors.

## Why this is deferred rather than open

**Fact — the four attributes are the family.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-41 is atomic per interpolation mode, with "output extents or scale factors, interpolation mode, coordinate transformation mode, and a boundary mode — four attributes that are commonly conflated and produce different tensors", and it says so explicitly: "The four attributes are the reason this is one family rather than one per mode, and omitting any of them makes the family underspecified."

**Fact — the numerical difficulty is in the coordinates, not the interpolation.** "Interpolation arithmetic is ordinary float arithmetic; the coordinate computation's rounding is where implementations diverge."

**Fact — its physical route is a gather.** D7 records "physical fallback is a gather with computed coordinates", and [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) places F-41 in its *covered only under a stated precondition* class, the precondition being "the gather its coordinates are read through". **Correction — 2026-08-10.** The filing-time sentence that followed treated that gather as live work under [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) owning the tensor-data-derived index class under Q-SHAPE-007. That claim is false against the present tree: the gather *semantic* family was delivered 2026-08-07 as `tiler::gather-f32@1` under ADR 0107 (ticket `status: done`), with semantic registration and reference evaluation live. What remains open is the index-layer / scheduled access representation and the physical backend route — successors [`revise-adr-0108-with-a-complete-data-dependent-index-vertical`](revise-adr-0108-with-a-complete-data-dependent-index-vertical.md) (`awaiting-decision`) and [`emit-the-indirect-gather-on-metal`](emit-the-indirect-gather-on-metal.md) (`blocked`) — so F-41's physical fallback still inherits O-08's uncovered physical half. The frontmatter dependency on the closed admit ticket stays as the historical chain link; it no longer names the remaining lowering gate.

**Inference — the dependency is real rather than topical.** Without an index-layer admission and physical gather route the family has no physical lowering, so scoping it first would produce a signature with no backend route. The semantic gather family alone does not discharge that gate.

## Activation trigger

A named image, signal, or vision workload requires resampling. The pinned language-model track does not reach it, and the roadmap's own candidate-track table records image and signal pipelines as "Not filed".

## What the work would be, when it starts

State all four attributes as canonical fields with their admissible values, and — the part that decides correctness — pin the coordinate computation's rounding rather than leaving it to the realization, since that is where implementations diverge. Then the exact-coordinate-then-interpolate oracle, and the gather-with-computed-coordinates lowering expressed over whatever access class the remaining gather-route work admits, rather than a second indirection.

## Explicit non-goals

- The indirect access class and physical gather route, which the gather successor tickets own.
- One family per interpolation mode, which the taxonomy's own reasoning rejects.
- A boundary mode implemented as a padding family. Reflect, edge, and wrap need the piecewise access class, and borrowing a pad here would hide that.

## Closes when

The family has all four attributes canonical, a pinned coordinate rounding, an exact oracle, and a lowering expressed over the admitted indirect access class — or is recorded as consumer-owned with the derivation.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-34** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-41 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No image, signal, or vision workload is filed; the roadmap's candidate-track table records that class as "Not filed", and the only live conformance track is language-model inference. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** The active consumer work remains language-model/conformance work; no image, signal, or vision workload names resize, grid sampling, or an affine warp. Gather/index representation work alone does not fire the workload half of this trigger.
- 2026-08-10 — **not fired.** Workload half still unsatisfied: no image, signal, or vision resampling workload filed; roadmap candidate-track "Image and signal pipelines" remains "Not filed". Semantic gather delivery (`tiler::gather-f32@1`) does not fire this trigger. Resampling family key remains unregistered.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it ~~returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output.~~ **Correction — 2026-08-22 (the total is withdrawn, not re-corrected).** The struck wording is kept as the claim it retired, not as a live instruction. Three things are wrong with it. Its `[a-z0-9-]+` class silently drops the five underscore-bearing MX keys — `tiler::mxfp4_e2m1@1`, `tiler::mxfp6_e2m3@1`, `tiler::mxfp6_e3m2@1`, `tiler::mxfp8_e4m3@1`, and `tiler::mxfp8_e5m2@1` — so the corrected class `[A-Za-z0-9_-]+` returns **55** here rather than 50. The figure is scope-bound rather than absolute: that same corrected pattern returns 63 under `crates/` and 72 across the repository outside `tickets/`, so the directory argument decides the answer as much as the tree does. And it was never a key population at all — governed keys are constructed, not spelled, by helpers such as `governed_op("assemble-strict-affine")` in `crates/tiler-ir/src/semantic/quantization.rs`, so `tiler::assemble-strict-affine@1` and `tiler::quantize-strict-affine@1` are governed keys carrying no bare literal and are absent from all 55. A total that has drifted 46 to 47 to 50, that moves whenever any unrelated dtype is admitted, and that has never once moved for the event this trigger watches is not this trigger's change-detector; the key-named check below replaces it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the geometric resampling family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(resample|resize|interpolate|grid-sample|upsample)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::resample-bilinear-f32@1` to that census makes the same command print `tiler::resample-bilinear-f32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
