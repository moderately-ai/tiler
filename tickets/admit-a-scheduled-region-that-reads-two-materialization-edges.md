---
id: admit-a-scheduled-region-that-reads-two-materialization-edges
title: Admit a scheduled region that reads two materialization edges
status: todo
priority: p3
dependencies: []
related: [admit-a-staged-family-that-reads-a-materialized-intermediate, admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner]
---
## User-visible outcome

`rms_norm(matmul(a, b), a)` compiles instead of refusing with no feasible plan — its consuming stage reads the occurrence's operand edge *and* the value the producing stage handed it, which is two materialization edges into one scheduled region.

## Where the wall is

**Fact.** `reads_bind_boundary_tensors_in_order` (`crates/tiler-ir/src/schedule/builder.rs`) admits at most one `TensorRole::Intermediate` read, correctly, because that role carries no ordinal: with two edges into one region nothing says which access binds which.

**Fact.** The compiler declines before proposing such a region rather than emitting one the verifier would reject as invalid compiler output. `physical::root_mean_square_scale_plan` destructures the recognized operand run as two `BoundaryRead::Input` operands and answers `None` for anything else, so `spell_staged` reports `region-staged-family-unspellable` and the compilation refuses under `NoFeasiblePlan`. `crates/tiler-compiler/tests/staged_family_over_a_materialized_intermediate.rs` measures that, with the two-declared-input normalization beside it as the control.

**Fact.** Recognition is *not* the wall. [`admit-a-staged-family-that-reads-a-materialized-intermediate`](admit-a-staged-family-that-reads-a-materialized-intermediate.md) landed the recognized shape: `NormalizedStaged::operand_reads` carries the operand's boundary role and `NormalizedStaged::producer` carries the shape whose regions write the edge, and `request::tests::a_staged_family_reading_a_materialized_intermediate_is_recognized` asserts both halves and the physical `None` beside them.

## Boundaries, read before starting

This is the sibling of [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md), which that ticket explicitly excludes ("different-edge pairs still refused by name"). Both need the same thing and neither may take it alone: **an ordinal on `TensorRole::Intermediate` is a public boundary** (ADR 0074 5b total maps in three compiler files) — draft plus acceptance node, never self-accepted — **and an identity step**: `push_tensor_role` writes a bare tag today, so any payload moves every intermediate-touching region, `tiler.schedule.v5` steps, and every pin recomputes, executed completely or not started. Decide with that ticket whether one step carries both widenings or whether one lands first; two independent steps of the same domain cannot compose.

The cover side is unread and must be read before the shape is chosen: whether cover enumeration places two edges into one region at all, and whether `region.rs`'s synthetic-intermediate record can name a second producer for one consuming stage — the same record [`carry-a-multi-reader-intermediate-through-region-formation`](carry-a-multi-reader-intermediate-through-region-formation.md) widens from the other direction.

## Closes when

`rms_norm(matmul(a, b), a)` compiles and bit-matches `tiler-reference`, a region reading two edges is verified with the mis-bound pairing still refused by name, the identity step is complete with every pin enumerated and recomputed on the merged tree, and the acceptance node parks.
