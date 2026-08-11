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

## Worker evidence — 2026-08-11

**Fact audit at base `4aaccb8206a922a940aacb06d53d7614889d089f`.** Both Facts above are verified after reading `crates/tiler-compiler/src/request.rs`, `crates/tiler-compiler/tests/composed_family_recognition.rs`, and `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs` in full. The existing public `reverse(silu(a))` row is a same-region computed operand, while the new subject is the direct mapped-only read of a contraction result the compiler otherwise materializes; no existing test pinned that subject. No ticket Fact needed repair.

**Delivered boundary.** `recognize_structural_read` now states both current paths rather than claiming a user-visible staged-operand admission: direct `reverse(contract(a, b))` reaches `structural-operand` before the producer is a staged leaf, while a dense read that first discovers the edge makes the mapped occurrence a second unordinalled staged read and `record_leaf` returns `structural-access-conflict`. The public compile regression runs the direct case under all five named F32 contracts exercised by this boundary suite beside the bare contraction, which compiles under the identical requests.

**Subject perturbation.** The regression's reindex alone was replaced with `F32Silu` over the same contraction result while its refusal assertion stayed unchanged. The first contract failed with `left: Ok(())` and `right: Err(UnsupportedCapability { rule: "structural-operand" })`; the reindex was restored before the final checks. This also establishes that the assertion reaches the changed occurrence before any contract-dependent later feasibility can hide it.

**Independent-review correction — 2026-08-11, exact candidate `2332cd8cf9690e1ca96dd38b75fcd038e7cde5ab`.** The candidate called those five named F32 points “all five statable numerical contracts,” and the adjacent test comment called them every contract a caller can state. Both population claims were false: callers can compose many other F32 dimension vectors and BF16 contracts also exist. The repaired wording names only the five F32 points this suite actually runs. Review otherwise found no defect in the refusal, control, perturbation, or boundary description.
