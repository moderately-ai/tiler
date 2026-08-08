---
id: repair-the-shape-records-after-sourced-semantic-result-shapes
title: Repair the shape records after sourced semantic result shapes
status: in-progress
priority: p1
dependencies: [correct-the-symbolic-coefficient-era-index-vocabulary-claims]
related: [repair-the-records-the-sourced-semantic-shape-falsifies]
scopes: [research/shapes]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, shapes, correction]
claimed_from: todo
assignee: w-terra-shapes
lease_expires_at: 1786210891
---
## Why this exists

The earlier [`repair-the-records-the-sourced-semantic-shape-falsifies`](repair-the-records-the-sourced-semantic-shape-falsifies.md) ticket is complete and records an older transition. It cannot own newly discovered drift as an active work lane. The coefficient-era audit found present-tense research and roadmap claims that still say inferred semantic results or `ValueFact` are static after sourced result-shape inference landed.

## Starting evidence, stale until re-read at this ticket's base

- `docs/research/shapes/symbolic-semantic-extents.md`, anchors `The Fact's narrow half survives`, `No symbol reaches an INFERRED RESULT`, and `keeping ValueFact on a fixed Shape`, retains the former boundary.
- `docs/roadmap.md`, anchor `a symbolic contracted extent is not reached either`, derives a current contraction limitation from a static value fact.
- `crates/tiler-ir/src/semantic/operation.rs`, anchor `pub struct ValueFact`, stores `SourcedShape`.
- `crates/tiler-ir/src/semantic/program.rs`, anchor `fn push_operation`, carries operation inference results into those facts.
- [`resolve-semantic-shape-inference-over-symbolic-extents`](resolve-semantic-shape-inference-over-symbolic-extents.md) records the implemented transition and its maturity boundary; it is evidence to verify, not a substitute for reading source.

The worker's first deliverable is a per-Fact verdict from the exact base. Read both documents in full, the complete implementation sites, the completed predecessor ticket, and governing accepted shape decisions. Re-derive the affected population rather than assuming these anchors are exhaustive.

## Outcome

Add dated corrections that distinguish the old static result boundary from the current source-bearing `ValueFact` and operation-inference path. For each operation family mentioned, say whether a symbolic result is constructible today and locate any remaining refusal at the family-specific schema, inference, bounds, or lowering layer instead of generalizing from one family to all semantic values.

This is documentation repair only. Do not introduce a new operation schema, inference rule, compiler capability, or public API. Preserve historical measurements and conclusions that do not depend on the retired premise.

## Closes when

Every live research/navigation claim derived from fixed `ValueFact::shape` is classified and corrected or supported; the contraction row no longer calls a general static-value boundary its blocker; related historical tickets are not presented as active owners; `make citations`, `tkt lint`, and `git diff --check` pass; and `tkt guard` shows no undeclared scope.
