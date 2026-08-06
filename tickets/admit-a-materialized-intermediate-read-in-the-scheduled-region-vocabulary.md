---
id: admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary
title: Admit a materialized-intermediate read in the scheduled-region vocabulary
status: todo
priority: p2
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, admit-a-reduction-over-a-declared-input-tensor, admit-multi-input-tensors-in-the-scheduled-region-vocabulary, accept-the-public-compiler-facade-boundary]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, schedule]
---
## User-visible outcome

A scheduled region whose elementwise body reads a value an earlier region materialized is expressible in `tiler_ir::schedule`, so a `producer -> intermediate -> elementwise epilogue` chain has regions to be assembled from. That is what `matmul(a, b) * 2.0`, `sum(x * x) * scale`, and the copy stage a published-and-consumed intermediate needs all bottom out on.

## Why this exists

`admit-elementwise-epilogues-over-a-materialized-intermediate` set out to build that chain in `tiler-compiler`. It cannot be done there, and the ticket was filed on the opposite premise — that the wall was "the physical layer's, not the schedule IR's", because `TensorRole::Intermediate` is a per-region role. The role *is* per-region. What forbids the chain is not the role but the access contract each scalar-program family declares around it, and those contracts live in `crates/tiler-ir/src/schedule/builder.rs`.

**Measurement — worktree at base `fd1716c4`, 2026-08-06, the pinned nightly.** Three hand-built regions submitted to `ScheduledRegionBuilder::build`, each paired with a control differing only in one tensor role:

| Region | Role under test | Verdict |
| --- | --- | --- |
| `ScalarProgram::PointwiseF32`, one read | read `TensorRole::Intermediate` | refused, `NumericalOrAccessRefinement` (control at `Input { ordinal: 0 }` verifies) |
| `ScalarProgram::StrictSerialSum`, `ReductionTopology::Serial` | write `TensorRole::Intermediate` | refused, `NumericalOrAccessRefinement` (control writing `Output` verifies) |
| `ScalarProgram::StrictTensorContraction` | write `TensorRole::Intermediate` | **admitted** — this half needs no widening |

Retained as `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs`; reproduce with `cargo nextest run -p tiler-compiler --test materialized_intermediate_epilogue_wall`. Every one of its five assertions was perturbed and observed failing before the file was committed.

**Fact — the two refusing rules, read from the verifier.** `verify_pointwise_region` computes `ordinals_bind_in_order` and requires read access `i` to be `TensorRole::Input { ordinal: i }` at *every* position, for both the `f32` and `bf16` widths that share it. `verify_access_and_semantics` admits a `StrictSerialSum` under a `ReductionTopology::Serial` only in the arm guarded by `read.tensor == TensorRole::Intermediate && write.tensor == TensorRole::Output`; the multi-pass partial arm is the one place a fold writes an intermediate, and it is a different topology declaring a split.

**Fact — the compiler cannot route around it by binding differently.** A region could declare `TensorRole::Input { ordinal }` for the read and let program assembly bind a temporary there only if `tiler_ir::program::ValueRole::fills` allowed it. It does not: `(Temporary, Input { .. })` is a refused pair at `crates/tiler-ir/src/program/model.rs:182`, and `KernelProgramBuilder::push_stage`'s `check_stage_accesses` is where that bites. So the escape hatch is closed independently, a second time, in `tiler-ir`. Already pinned by `a_published_output_value_cannot_fill_an_intermediate_buffer` in `crates/tiler-compiler/tests/multi_output_boundary.rs`.

**Inference — this is the multi-input shape again.** `admit-multi-input-elementwise-programs-at-the-compiler-boundary` hit the identical structure: the recognizers were where the refusal was *observed*, and the vocabulary that made the shape inexpressible lived a crate down. That ticket's resolution — file the `tiler-ir` widening, make the compiler-side admission its dependent rather than its peer — is the one being repeated here.

## Boundaries

- **A widening of an admitted access, not a relaxation.** A pointwise region reading an intermediate must still prove its bounds, its ownership, and its map admissibility exactly as an input-reading one does. The `ordinals_bind_in_order` correspondence exists so that a consumer binding buffers positionally cannot bind the wrong one; whatever replaces it must answer the same question for a mixed read list, not stop asking it.
- **`TensorRole::Intermediate` carries no ordinal, and that is the design problem.** `Input` distinguishes its tensors by ordinal; `Intermediate` does not. `CoverAssembly::from_plan` already refuses more than one intermediate read per dispatch under `cover-intermediate-read-attribution` for exactly that reason. A region reading two intermediates has nothing to say which edge binds which access, so either the widening admits one intermediate read and refuses a second by name, or it gives the role an ordinal — and the second is an identity-domain change reaching `encode_identity`, every schedule identity, and every kernel identity.
- **Whether the serial-sum write widens is a separate question from the pointwise read.** `sum(x * x) * scale` needs both; `matmul(a, b) * 2.0` needs only the read, because the contraction producer already composes. Splitting them is admissible and the contraction shape is the smaller first slice.
- **The identity consequences are the expensive part.** A widened access contract changes what `encode_identity` can encode; `admit-multi-input-tensors-in-the-scheduled-region-vocabulary` is the precedent for how far that reaches (it carried scopes into the artifact, Metal, build, and frontend crates). Size that before scaffolding, and do not step an identity domain half-way.
- **Do not synthesize a copy to satisfy the verifier.** Staging a materialized copy of a value a region could read directly adds an observable rounding boundary the caller's program never asked for — the same reason `admit-a-reduction-over-a-declared-input-tensor` refuses that shortcut.

## Closes when

A pointwise scheduled region reading `TensorRole::Intermediate` verifies, with its bounds, ownership, and map obligations discharged exactly as the input-reading one's are; the positional-binding obligation `ordinals_bind_in_order` carried is restated for a mixed read list and observed refusing a region that violates it; a region reading two intermediates refuses by name rather than binding one edge twice; and `materialized_intermediate_epilogue_wall.rs`'s pointwise assertion is inverted in the same change that lifts it, rather than deleted.
