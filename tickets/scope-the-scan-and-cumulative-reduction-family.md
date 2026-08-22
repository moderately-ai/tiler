---
id: scope-the-scan-and-cumulative-reduction-family
title: Scope the scan and cumulative reduction family
status: deferred
priority: p3
dependencies: []
related: [implement-parallel-reduction-strategies, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, reductions, scan, deferred]
---
## User-visible outcome

A cumulative sum or product is a family whose legal realizations are stated up front, so that nobody discovers after implementing a work-efficient parallel scan that the contract never permitted it.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-31 is atomic and "a scan is not a reduction with retained partials, because its result shape and its parallel realization both differ". It carries input, accumulator, and result types as three fields exactly as F-28 does, plus a scanned axis and exclusive-or-inclusive and reversed flags.

**Fact — the sharpest sentence in the row is a legality statement, not a performance one.** "An associative-scan realization reassociates by construction, so a scan under a non-reassociating contract has only the serial realization." [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) agrees from its own side, placing F-31 in the *covered — direct fold or nested loop* class where "the profile must have the *serial* form; the tree, split, multi-pass, GEMM, im2col, and Winograd forms are alternatives beside it and never substitutes".

**Inference — that makes the scan the cheapest available test of a rule the corpus states everywhere and has exercised once.** [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md) holds reassociation and permutation as independent permissions each requiring both an operation capability and an effective numerical permission. A scan is the family where the *only* interesting realization consumes one of them, so admitting it under a strict contract delivers exactly the serial form and nothing else — which is correct, and which a reader expecting a parallel scan will read as a defect unless the record says so first.

**Fact — segmented and windowed scans are separate families and are unsupported**, which the taxonomy states directly; this track carries neither.

## Activation trigger

A named workload requires a cumulative reduction along an axis. Note the anti-trigger: a *reduction* over an axis does not fire it, because a scan's result shape is the operand's rather than the reduced one, and reusing a reduction for it would be a different operation.

## What the work would be, when it starts

The key, the scanned axis, the exclusive/inclusive and reversed flags, the three separately declared dtype fields, the serial prefix fold as the oracle, and — stated before any implementation — the legality table: which realizations are available under a contract that grants reassociation and which under one that does not, with the serial form named as the unconditional baseline. Then say what an empty and a single-element scanned axis produce, which the reduction's empty-domain rule does not answer because a scan has no empty result.

## Explicit non-goals

- Segmented and windowed scans, which are separate families.
- Any parallel scan admitted under a non-reassociating contract; that is the case this track exists to refuse.
- Reusing F-28's empty-domain rule, which is about a reduced axis and not a retained prefix.

## Closes when

The family has a key, a serial oracle, a stated legality table separating the serial realization from the reassociating ones, and an answer for the empty and singleton axis — or is recorded as unneeded with the consumer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-27** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-31 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires a cumulative reduction; the pinned workload's only axis-wise accumulations are the contraction's fold, the normalization's squared sum, and the softmax's two folds, none of which retains a prefix. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** No selected workload requires a cumulative prefix result. The additional staged reduction and tree-topology work still produces one reduced result rather than retaining every prefix, so it does not activate this semantic family.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it ~~returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output.~~ **Correction — 2026-08-22 (the total is withdrawn, not re-corrected).** The struck wording is kept as the claim it retired, not as a live instruction. Three things are wrong with it. Its `[a-z0-9-]+` class silently drops the five underscore-bearing MX keys — `tiler::mxfp4_e2m1@1`, `tiler::mxfp6_e2m3@1`, `tiler::mxfp6_e3m2@1`, `tiler::mxfp8_e4m3@1`, and `tiler::mxfp8_e5m2@1` — so the corrected class `[A-Za-z0-9_-]+` returns **55** here rather than 50. The figure is scope-bound rather than absolute: that same corrected pattern returns 63 under `crates/` and 72 across the repository outside `tickets/`, so the directory argument decides the answer as much as the tree does. And it was never a key population at all — governed keys are constructed, not spelled, by helpers such as `governed_op("assemble-strict-affine")` in `crates/tiler-ir/src/semantic/quantization.rs`, so `tiler::assemble-strict-affine@1` and `tiler::quantize-strict-affine@1` are governed keys carrying no bare literal and are absent from all 55. A total that has drifted 46 to 47 to 50, that moves whenever any unrelated dtype is admitted, and that has never once moved for the event this trigger watches is not this trigger's change-detector; the key-named check below replaces it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the scan and cumulative reduction family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(scan|cumsum|cumprod|cumulative|prefix-sum)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::cumsum-f32@1` to that census makes the same command print `tiler::cumsum-f32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
