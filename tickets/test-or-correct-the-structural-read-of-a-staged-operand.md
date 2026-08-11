---
id: test-or-correct-the-structural-read-of-a-staged-operand
title: Test or correct the structural read of a staged operand
status: in-progress
priority: p3
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, move-the-structural-row-to-r6-and-retire-its-backend-residual]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, doc-claim, structural, tests]
claimed_from: todo
assignee: terra-structural-staged-read
lease_expires_at: 1786425351
---

## The claim, and what the source actually does

**Fact — corrected 2026-08-09.** The positive admission sentence is still present under source anchor `The operand must be a value this walk reads rather than computes`, but it is false as a user-visible admission. A mapped-only `reverse(folded)` reaches `recognize_structural_read` on the first walk before any staged leaf has been discovered, so `if !leaves.is_leaf(*operand)` returns `structural-operand`. If a dense occurrence first discovers the materialized producer, replay does mark it as a staged leaf, but the mapped occurrence is then a second read of that staged value and `record_leaf` returns `structural-access-conflict`.

**Fact — corrected 2026-08-09.** The combination is not wholly untested. The request regression under anchor `let staged = |mapped: bool|` builds `s * reverse(s)` with `s = sum(a, axis 1)`: the dense neighbour recognizes as an epilogue, while the mapped second read returns `structural-access-conflict`. What remains unpinned at the public compile boundary is the first case: a direct mapped-only structural occurrence over one materialized result returns `structural-operand`.

A doc comment is a claim the next worker acts on (AGENTS.md), and this one makes unreached work look reachable: a reader planning `reverse(matmul(a, b))` would conclude the region vocabulary admits it today.

## The work

Correct the source comment to name both current refusal paths rather than claiming admission. Add a public `compile()` regression beside `contraction_with_epilogue`: a direct reindex such as `reverse(contract(a, b))` must refuse under `UnsupportedCapability { rule: "structural-operand" }`, while the bare contraction remains admitted. This is a refusal regression, so it does not need a bit comparison.

Perturb the subject, not the expectation: replace the reindex with one `F32Silu` occurrence over the same contraction result. That neighbour is an admitted epilogue and must make the refusal assertion fail with `left: Ok(())`; restore it before the final gates.

## Closes when

The doc comment states both tested refusal paths, the direct mapped-only public regression pins `structural-operand` with its admitted bare-contraction neighbour, and any desired admission remains separate from this correctness repair.
