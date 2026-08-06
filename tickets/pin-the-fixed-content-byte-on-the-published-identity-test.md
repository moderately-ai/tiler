---
id: pin-the-fixed-content-byte-on-the-published-identity-test
title: Pin the fixed-content byte on the published-identity test
status: todo
priority: p3
dependencies: []
related: [attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget, decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifacts, measurement]
---
## User-visible outcome

An unexplained movement of the artifact envelope's fixed content fails a test with a superseded-value ledger, instead of being discovered days later by a research sweep — the budget answer [the manifest-growth record](../docs/research/artifacts/manifest-fixed-content-growth.md) recommends, implemented.

## Why this shape, from the record's own measurement

**Measurement.** Across the 107 landings the attribution swept, a fixed-content byte pin would have fired **3 times, and all three were identity-domain steps already rebaselining `the_standard_metal_path_publishes_its_recorded_identities`** (crates/tiler-build) — so the pin adds no rebaseline event to the workflow, only one number to a file that was already moving whenever it would fire. That test already pins two identities with a superseded-value ledger and regeneration mechanics; the fixed-content byte joins them in the same idiom.

**The counterpoint the record demonstrated, carried rather than dropped.** A fixed-fixture pin is blind to program-size growth: `36d05128` raised the governed `semantic_operations` budget 8 → 62 — admitting programs ~2.8× past the 1 MiB embedding ceiling — and moved the fixture by zero. The blind spot's coverage is [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md), which the record ranks first; this pin is the cheap second half, not a substitute.

## The work

Add the fixed-content byte count of a zero-object envelope of the standard Metal path's program to `the_standard_metal_path_publishes_its_recorded_identities`' pinned values, with the ledger paragraph stating what the number is, why it moves only at encoding changes, and the regeneration mechanics the test's existing pins use. Watch it fail under a deliberate perturbation (any one-byte manifest addition) before trusting it. If [`decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest`](decide-whether-the-manifest-carries-the-identity-preimage-or-its-digest.md) lands first, the pin's initial value is taken after that step rather than before it.

## Closes when

The pin exists with its ledger, was watched failing, and the record's Section 6 recommendation row points at it as implemented.
