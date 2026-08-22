---
id: scope-the-ordered-search-family
title: Scope the ordered search family
status: deferred
priority: p3
dependencies: []
related: [scope-the-ordering-and-rank-selection-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, search, deferred]
---
## User-visible outcome

A binary search over sorted data is a family whose sortedness precondition is a *declared and validated* value assumption, not an undocumented expectation that turns a wrong answer into a plausible one.

## Why this is deferred rather than open, and why it is not grouped with sorting

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-39 is atomic, two operands and one result, data and needle of one identical type with an index result, one side attribute selecting left or right insertion, and one D5 sentence that separates it from everything else in its group: it "requires the data to be sorted, which is a **value assumption** rather than a shape constraint". Its D8 routes that assumption: "A sortedness assumption declared but unproved falls under [ADR 0021](../docs/decisions/0021-validated-value-assumptions.md) and needs proof or runtime validation."

**Inference — that is why it is not in the ordering track.** Sort and top-k *produce* an order and owe a total order and a tie-break; this family *consumes* an order it cannot see and owes a validated precondition. The implementations differ too — a sort, versus a binary search parallel over the needles — so grouping them would put one track's correctness argument on two families and one implementation on two physical shapes.

**Inference — the failure mode is the reason to state the assumption rather than assume it.** A binary search over unsorted data returns an index, in range, of the right type, for every input. Nothing downstream can tell. That is the same silent-wrongness shape the corpus concentrates scrutiny on, and it is why this family's deliverable is the precondition rather than the search.

## Activation trigger

A named workload requires an ordered search — a bucketing step, an interpolation table lookup, or a quantile evaluation.

## What the work would be, when it starts

The key, the identical-type data and needle admissible set, the index result type, the side attribute, the binary-search oracle, and the sortedness assumption expressed as an ADR 0021 value assumption with its two discharge routes stated — proved where the producer is a sort the compiler can see, and a typed host-side pre-dispatch validation where it is not. State the total order the assumption is *about*, which must be the one the ordering track selects, so the two families cannot disagree about what sorted means.

## Explicit non-goals

- Sorting itself, which is [`scope-the-ordering-and-rank-selection-families`](scope-the-ordering-and-rank-selection-families.md)'s.
- A search that silently defines a result for unsorted data. That is the failure this family exists to prevent.
- Interpolation on top of the search, which is a separate arithmetic family.

## Closes when

The family has a key, a binary-search oracle, and a sortedness precondition routed through ADR 0021 with both discharge routes stated and the refusal watched firing — or is recorded as unneeded with the consumer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-32** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-39 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires an ordered search. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** No bucketing, interpolation-table, quantile, or other ordered-search consumer has entered a semantic program. The newer value-domain provenance machinery still has no sortedness predicate or validation route.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it ~~returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output.~~ **Correction — 2026-08-22 (the total is withdrawn, not re-corrected).** The struck wording is kept as the claim it retired, not as a live instruction. Three things are wrong with it. Its `[a-z0-9-]+` class silently drops the five underscore-bearing MX keys — `tiler::mxfp4_e2m1@1`, `tiler::mxfp6_e2m3@1`, `tiler::mxfp6_e3m2@1`, `tiler::mxfp8_e4m3@1`, and `tiler::mxfp8_e5m2@1` — so the corrected class `[A-Za-z0-9_-]+` returns **55** here rather than 50. The figure is scope-bound rather than absolute: that same corrected pattern returns 63 under `crates/` and 72 across the repository outside `tickets/`, so the directory argument decides the answer as much as the tree does. And it was never a key population at all — governed keys are constructed, not spelled, by helpers such as `governed_op("assemble-strict-affine")` in `crates/tiler-ir/src/semantic/quantization.rs`, so `tiler::assemble-strict-affine@1` and `tiler::quantize-strict-affine@1` are governed keys carrying no bare literal and are absent from all 55. A total that has drifted 46 to 47 to 50, that moves whenever any unrelated dtype is admitted, and that has never once moved for the event this trigger watches is not this trigger's change-detector; the key-named check below replaces it. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Census sized from the family's own key — 2026-08-22; no verdict re-decided here.** The governed-key *total* this log carried at 46, then 47, then 50 is withdrawn rather than corrected a fourth time, because no command a reader can run derives it: keys are built by constructors rather than written as literals, and registration is spread across ten `register_standard_*` helpers that no single command walks. The companion figure in the same entries — *the eighteen registered operation keys* — is stale by one and is withdrawn with it. That one is genuinely derivable, and the derivation is `rg -o -N --no-filename 'pub fn [a-z0-9_]+_op\(\)' crates/tiler-ir/src/semantic/ | sort -u | wc -l`, which reports **19** at this base; it counts unique occurrences of the anchored form rather than matching lines, and adding one `pub fn …_op()` moves it to 20. The single further `_op()` definition in that directory is excluded deliberately: `external_identity_op` is not `pub` and its key is `OpKey::new("acme", "identity", 1)`, an external-namespace test fixture rather than a governed key. What this trigger actually waits on is the ordered search family's own key, which does not exist at this base. The check is therefore stated over the key instead of over a total: `rg -o -N --no-filename 'tiler::[A-Za-z0-9_-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u | rg -w '(search|searchsorted|binary-search|bucketize|digitize)'` prints nothing at this base, and any output at all is the changed answer. It was run before being written down, and shown able to say *no*: appending the synthetic literal `tiler::searchsorted-f32@1` to that census makes the same command print `tiler::searchsorted-f32@1`. The token list is a screen over the family's operation names and not a proof of absence — a key spelled outside those tokens would not be caught, so a reader who finds the screen empty while the family is plainly registered should widen the tokens rather than trust the silence. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
