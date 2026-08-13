---
id: bound-canonical-entry-ordinal-lookup-cost-in-loader-preflight
title: Bound canonical entry ordinal lookup cost in loader preflight
status: in-progress
priority: p2
dependencies: [select-executable-variants-across-registered-backend-families]
related: [accept-the-loader-variant-eligibility-vocabulary]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, performance, research]
claimed_from: todo
assignee: worker-bound-ordinal
lease_expires_at: 1786587491
---
## User-visible outcome

Measure and, if warranted, remove repeated canonical-entry ordinal scans from host-side loader preflight without changing routing semantics, refusal order, artifact identity, or kernel behavior.

## Why this exists

**Inference from source structure — re-measure before optimizing.** Eligibility is commonly summarized as one `O(variants + entries)` host walk, but `CanonicalEntryOrdinals::of` builds stage-key vectors and recomputes prior-entry bases for each assessed variant, while `of_entry` linearly searches entries. The resulting host-side work can approach `O(V^2 + sum(E_v^2))` in portfolio shape even though the device-independent comparisons themselves are linear.

This is Tiler runtime overhead, not kernel runtime. It occurs during preflight before device execution and is independent of the accepted no-fallback semantics.

## Required research boundary

- Re-derive exact decoded-artifact bounds and current ordinal construction/consumption sites.
- Construct a bounded portfolio/entry matrix and measure or count comparisons/allocations without device timing.
- Decide whether an already-verified ordinal index can be reused or built once without creating a second authority.
- Preserve canonical ordinal meaning and every refusal/selection ordering.

## Stop conditions

Stop before implementation if the proposed cache/index becomes artifact state, changes identity bytes, weakens decode verification, or needs a new public API. File the corresponding authority/identity decision instead.

## Non-goals

No kernel-performance claim, backend fallback, cost-based runtime selection, or change to the public eligibility vocabulary.

## Fact audit — 2026-08-12 at `611fefee15d8878b9458bd860d09490ec736a17f`

Every claim below was re-read at this base. Citations are searchable anchors, not line numbers.

1. **Eligibility is commonly summarized as one `O(variants + entries)` host walk.** **Imprecise as a written formula, verified as the surface walk.** No crate source writes the big-O string. What exists is the walk `select_variant` documents — packaged variants once, then every entry of each assessed variant in execution order — which `select-executable-variants-across-registered-backend-families` restates as "walks the packaged variants once" and "walks **every entry in execution order**". That surface is linear in assessed variants plus their entries. It is not the cost of constructing the canonical ordinals those entries then consume.

2. **`CanonicalEntryOrdinals::of` builds stage-key vectors and recomputes prior-entry bases for each assessed variant.** **Verified at the pre-change body.** `variant_eligibility` called `CanonicalEntryOrdinals::of(&self.decoded, variant)` after the assessed-profile filter. `of` summed `decoded.variants().take(variant.routing_rank()).map(|earlier| earlier.entries().len())` and collected `variant.entries().map(DecodedEntry::stage_key)`. Assessed-profile failure never reached `of`. `select_variant` still stops at the first eligible guard that holds, so the number of assessed variants is at most `V`.

3. **`of_entry` linearly searches entries.** **Verified at the pre-change body.** `stage_keys.iter().position(|key| *key == entry.stage_key())`.

4. **Host-side work can approach `O(V^2 + sum(E_v^2))` in portfolio shape.** **Verified as the worst-case count, with the following precision.** The `V^2` term is preceding-variant visits while re-summing bases: `V(V-1)/2` when every variant is assessed, and each `entries().len()` is `O(1)`. The `sum(E_v^2)` term is stage-key equality comparisons: `E(E+1)/2` per fully-walked variant when execution order matches canonical order, and up to `E^2` when the probe order is the reverse permutation. Early exits cut the count: `UnsupportedRepresentation` and `PayloadProfile` skip remaining `of_entry` calls on that variant. The formula is therefore the case where every variant reaches the dtype check, not the typical fixture.

5. **Device-independent comparisons themselves are linear.** **Verified.** Per assessed variant: one profile classification. Per assessed entry that reaches it: backend/representation pair, payload-profile classification, `classify_dtype`, and `binary_search_by_key` on the delivered-realization bindings. Those comparisons do not contain the ordinal reconstruction.

6. **The work occurs during preflight before device execution.** **Verified.** `preflight` and `prepare` both call `select_route`, which calls `select_variant`, which calls `variant_eligibility` before any guard, route requirement, deferred predicate, payload-object check, or device stage.

7. **Independent of the accepted no-fallback semantics.** **Verified.** Eligibility is a filter against one stated `ExecutionEnvironment`. There is no backend retry. The ordinal work is host bookkeeping for the dtype-binding lookup, not a kernel or selection-policy change.

8. **Decoded-artifact bounds (missing from the ticket, load-bearing for the warrant).** **Verified.** `MAX_ARTIFACT_VARIANTS` is 64 and `MAX_VARIANT_ENTRIES` is 4_096 in `tiler_artifact::program`. A decode admits a portfolio at those bounds. `stage_keys_collide` refuses two stages of one variant with equal keys; `check_ordered` on `OrderedSubject::Entry` refuses an unsorted or repeated stage-key table. `DecodedEntry`'s within-variant index is the canonical table position, but it is a private field of a public type — publishing it would be a new public API and is therefore out of this ticket.

## Measurement — host bookkeeping only, no device

Population: every `(V, E)` cell below, with every variant assessed through the dtype check and every entry probed once. Counts are stage-key `Ord` comparisons and prefix visits, not wall time. The live function `canonical_stage_index` is the subject: wrapping its keys in a counting `Ord` and replacing `binary_search` with `position` is the perturbation that reddens the bound test.

| V | E per variant | Prefix visits (old) | Prefix visits (new) | Stage-key comparisons, exec = canon (old, exact) | Logarithmic comparison bound (new) |
| --- | --- | --- | --- | --- | --- |
| 1 | 1 | 0 | 1 | 1 | 1 |
| 1 | 2 | 0 | 1 | 3 | 4 |
| 2 | 2 | 1 | 2 | 6 | 8 |
| 3 | 8 | 3 | 3 | 108 | 96 |
| 8 | 64 | 28 | 8 | 16,640 | 3,584 |
| 64 | 4_096 | 2_016 | 64 | 536,999,984 | 3,407,872 |

The last row is the decode-bound corner: `MAX_ARTIFACT_VARIANTS × MAX_VARIANT_ENTRIES`. Old stage-key work is `V × E × (E + 1) / 2`. The new bound is `V × E × (⌊log₂ E⌋ + 1)`. Allocations stay one `Vec` of `E` borrowed stage keys per assessed variant, plus one prefix `Vec` of `V` `u32`s.

Typical workspace fixtures (`V ≤ 3`, `E ≤ 2`) sit in the first three rows and would not by themselves warrant a change. The warrant is the admitted bound: a legal artifact can impose half a billion stage-key comparisons on a device-free walk.

**Perturbation of the subject.** Replacing `binary_search` with `position` in `canonical_stage_index` fails `ordinal_lookup_comparisons_stay_logarithmic_on_the_portfolio_matrix` at the first cell above the handful-of-entries threshold:

```text
V=3 E=8: 108 stage-key comparisons exceed the logarithmic bound 96
```

108 is the linear scan `3 × 8 × 9 / 2`. Restored to `binary_search` the same assertion is green.

**Already-verified index, reused rather than invented.** The prefix is the same accumulation `packaged_entry_positions` uses for the builder remap, read from `DecodedVariant::entries().len()` in routing order. The within-variant index is the decode-proved position in `DecodedVariant::entries`. Neither is stored on the artifact, neither enters identity bytes, and decode verification is unchanged. A public `DecodedEntry` accessor for the private table index was not added.

## Decision

Implementation is warranted at the decode bound. The change stays crate-private inside `tiler-runtime::load`: one prefix per `select_variant` walk, binary search over the already-sorted unique stage-key table. No public eligibility type, no artifact-state cache, no identity-byte change, no weakened decode check.

## What landed

- `packaged_entry_bases` derives the flat prefix once per selection walk.
- `canonical_stage_index` binary-searches the decode-proved stage-key table.
- Unit tests pin the prefix, the reverse-permutation lookup the live fixture cannot package, and the comparison bound on the matrix above.
- Routing semantics, refusal order, artifact identity, and the public eligibility vocabulary are untouched.

## Measurement boundary

Host-side comparison and allocation counts derived from the decoded artifact bounds and exercised on a synthetic key matrix. No device timing. No kernel-performance claim. No wall-clock claim about typical fixtures. The live adapter-route suite still cannot package a variant whose execution order differs from its canonical stage-key order; the reverse-probe unit test is the observation that suite lacks.
