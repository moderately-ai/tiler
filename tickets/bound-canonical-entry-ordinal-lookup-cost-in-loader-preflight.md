---
id: bound-canonical-entry-ordinal-lookup-cost-in-loader-preflight
title: Bound canonical entry ordinal lookup cost in loader preflight
status: todo
priority: p2
dependencies: [select-executable-variants-across-registered-backend-families]
related: [accept-the-loader-variant-eligibility-vocabulary]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, performance, research]
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
