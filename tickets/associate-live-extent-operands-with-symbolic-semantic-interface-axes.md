---
id: associate-live-extent-operands-with-symbolic-semantic-interface-axes
title: Associate live-extent operands with symbolic semantic interface axes
status: todo
priority: p0
dependencies: [admit-symbolic-extents-through-schedule-formation]
related: [admit-live-extent-operands-to-payload-indexing, accept-the-live-extent-artifact-envelope-row, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n]
scopes: [implementation/ir, implementation/artifact, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity, public-boundary, shapes]
---
## User-visible outcome

An artifact that declares a live input-axis operand is tied to the same symbolic axis in the semantic program and its carried shape environment. A fixed semantic interface cannot be executed under a different live shape while retaining the fixed program's coverage or identity.

## Exact gap and per-Fact audit at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`

- **Verified.** `crates/tiler-artifact/src/program/builder.rs` `read_semantic_interface` calls `static_interface_shape`, whose `SymbolicSemanticInterface` arm refuses every symbolic interface. Only a fixed `Shape` can be published today.
- **Verified.** The same file's `derive_extent_operands` proves that a kernel operand maps to a program input and that its axis is inside the fixed rank. It does not prove that the semantic axis is symbolic or that the operand names the program's carried `ShapeEnv` source.
- **Verified.** `crates/tiler-runtime/tests/adapter_route/fixture.rs` constructs `semantic_program()` with `input_shape() == Shape::from_dims([2, 3])`; `one_live_extent_payload_and_pipeline_indexes_dense_f32_at_two_n` then binds axis 1 to 14 and 15. The test passes while executing meanings outside that fixed semantic graph.
- **Verified.** The old draft `9a8f53c9` cannot repair this: it carried no artifact row, and the current split ports are strictly additive over it. The defect is in the live artifact's semantic authority, not a missing conflict port.

Reproduce the current wrong-positive with `cargo test -p tiler-runtime --test adapter_route one_live_extent_payload_and_pipeline_indexes_dense_f32_at_two_n -- --exact --nocapture`. The pass is failure evidence: a static `[2,3]` subject is accepted for `[2,14]` and `[2,15]` execution.

## Required work

- First make the current static-semantic/live-axis combination fail closed. Do not retain a compatibility path that lets a fixed semantic axis acquire a caller-selected extent.
- After [`admit-symbolic-extents-through-schedule-formation`](admit-symbolic-extents-through-schedule-formation.md) establishes the schedule carrier, associate each artifact extent operand with the exact source-bearing semantic input axis and the program's one shape environment. Reject a static axis, a foreign symbol/environment, an inferred or output axis, and a scheduled live axis not present in the semantic interface.
- Carry the source-bearing interface through every construction, validation, schema, coverage, and identity site required for one true semantic `[2,N]` artifact. Do not encode the bound value or specialize identity on 14/15.
- Re-audit whether `{ key, axis, value_type }` is sufficient envelope authority once the semantic source is present. If symbol identity or another public/schema field is required, produce the labelled draft and update Tom's packet; do not smuggle it into the accepted row.
- Remove the static `[2,3]` worked example once the symbolic subject exists. The example must exercise the general association rather than define it.

## Required evidence

- Static `[2,3]` plus a live operand on axis 1 refuses at artifact construction, with the subject perturbed and assertion unchanged.
- A semantic `[2,N]`, its schedule, kernel operand, artifact row, and shape environment all name one source. Swapped symbols/environments, a static axis, wrong axis, and missing source each refuse at their owning layer.
- Coverage and every semantic/artifact/payload identity are equal across bindings of 14 and 15, while a genuinely different symbolic program or a baked neighbour differs.
- Exact schema/domain blast radius, targeted IR/artifact/runtime tests, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Public boundary and stop condition

This is a semantic-interface, artifact-schema, and identity decision. Use Sol and independent review. If the existing `DecodedExtentOperand` row cannot identify the governing semantic source without widening a public type or schema, stop at a tested labelled draft and update [`accept-the-live-extent-artifact-envelope-row`](accept-the-live-extent-artifact-envelope-row.md) with a Pareto-complete packet before dependent implementation proceeds.

## Closes when

No fixed semantic graph can execute under a different bound extent, and one true symbolic semantic subject carries the same source through schedule, artifact, coverage, identity, and runtime binding.
