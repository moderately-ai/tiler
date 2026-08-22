---
id: scope-the-counter-based-random-generation-family
title: Scope the counter-based random generation family
status: deferred
priority: p3
dependencies: []
related: [scope-the-non-tensor-value-kinds-and-control-constructs, scope-the-effect-signature-opening, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, random, deferred]
---
## User-visible outcome

In-graph randomness, when it arrives, arrives as a **pure** family threading its state as an ordinary value — never as hidden state — and its algorithm is part of its identity rather than an implementation default.

## Why this is deferred rather than open, and why it is not an effectful family

**Fact — the classification is confident and it is the opposite of what the name suggests.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-43 is "Atomic, with the state threaded explicitly as an operand and a result", one operand and two results — an updated state and the generated tensor — and "**Pure**, because the state is a value rather than hidden mutable state". The taxonomy's conclusion 6 groups it with scatter: "Two families are pure despite reading as stateful... and both are pure for the same reason: the state is a value."

**Fact — the shape is taken from a primary source.** StableHLO's `rng_bit_generator` takes an `rng_algorithm` and an `initial_state` and returns both an `output_state` and an `output`, and states that "The output is guaranteed to be deterministic function of `initial_state`, but it is not guaranteed to be deterministic between implementations"; its `rng` operation is listed among the deprecated operations. The explicit-state form is what the ecosystem converged on, and it is the only form compatible with [Vision](../docs/vision.md)'s rule that a semantic program "has no hidden persistent state and no semantic loop across invocations".

**Fact — the one rule this family must not relax.** "An implementation-defined default algorithm is intentionally rejected; the algorithm is part of identity or the operation is not reproducible."

**Inference — this ticket exists partly to prevent a mis-filing.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Effectful and stateful operations` row names "hidden randomness" among its members. A counter-based generator is not hidden randomness and does not need the effect vocabulary; reading it into that row would make a pure family look like it waits on [Q-SEM-011](../docs/open-questions.md#q-sem-011--semantic-effects-and-resource-tokens). It does not.

## Activation trigger

A named workload requires randomness inside the program — dropout, in-graph sampling, or a stochastic rounding step. Inference-time sampling on the consumer side does not fire it, and the current conformance track samples outside the program by the L6 record's own boundary.

## What the work would be, when it starts

The key with the algorithm as an identity field; the state's own type and the extent that depends on the algorithm; two ordered results, which makes this an early exercise of the multi-result invariant the graph already states; the exact named-algorithm oracle; the per-element physical route; and the boundary the taxonomy leaves open and this work must not silently close — whether the state is a tensor or a distinct value kind, which is [`scope-the-non-tensor-value-kinds-and-control-constructs`](scope-the-non-tensor-value-kinds-and-control-constructs.md)'s question rather than this family's.

## Explicit non-goals

- Any effect vocabulary. This family is pure and widening `OperationEffect` for it would be a category error.
- A distribution transform, which is elementwise transcendental arithmetic with its own accuracy contract on top of the bit generator.
- An implementation-defined default algorithm.

## Closes when

The family has a key carrying its algorithm as identity, an exact oracle, two ordered results, and a stated position on whether its state is a tensor — or is recorded as unneeded with the consumer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-36** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-43 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires in-graph randomness; the pinned track is inference and samples on the consumer side. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** The selected language-model workload still has `attention_dropout = 0.0`, and token sampling remains outside the semantic program. No stochastic-rounding or in-graph sampling consumer exists, and no counter-based generator key is registered.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it ~~returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output.~~ **Correction — 2026-08-22 (the total is withdrawn, not re-corrected).** The struck wording is kept as the claim it retired, not as a live instruction. Three things are wrong with it. Its `[a-z0-9-]+` class silently drops the five underscore-bearing MX keys — `tiler::mxfp4_e2m1@1`, `tiler::mxfp6_e2m3@1`, `tiler::mxfp6_e3m2@1`, `tiler::mxfp8_e4m3@1`, and `tiler::mxfp8_e5m2@1` — so the corrected class `[A-Za-z0-9_-]+` returns **55** here rather than 50. The figure is scope-bound rather than absolute: that same corrected pattern returns 63 under `crates/` and 72 across the repository outside `tickets/`, so the directory argument decides the answer as much as the tree does. And it was never a key population at all — governed keys are constructed, not spelled, by helpers such as `governed_op("assemble-strict-affine")` in `crates/tiler-ir/src/semantic/quantization.rs`, so `tiler::assemble-strict-affine@1` and `tiler::quantize-strict-affine@1` are governed keys carrying no bare literal and are absent from all 55. A total that has drifted 46 to 47 to 50, that moves whenever any unrelated dtype is admitted, and that has never once moved for the event this trigger watches is not this trigger's change-detector; the key-named check below replaces it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the counter-based random generation family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(random|rng|philox|threefry|counter)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::philox-random-u32@1` to that census makes the same command print `tiler::philox-random-u32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
