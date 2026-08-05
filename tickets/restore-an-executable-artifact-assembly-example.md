---
id: restore-an-executable-artifact-assembly-example
title: Restore an executable artifact assembly example
status: todo
priority: p3
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [documentation, artifact]
---
## User-visible outcome

A reader following `tiler_artifact::program`'s and `tiler_artifact::proof`'s module walk-throughs is following code the gate compiles, rather than prose that may have drifted from the builder it demonstrates.

## Fact — what changed and why

`bind-stage-coverage-to-index-refinement-identity` made stage coverage proof-derived: `KernelProgramBuilder::push_stage` takes `CoveredOccurrence` records, and the only constructor for one takes a completed `tiler_ir::index::IndexRefinementReceipt`. Both module examples assemble a real kernel program in their hidden preamble, so both now need receipts, and both were marked `ignore` in that commit rather than left broken.

Two routes to a receipt exist and each is refused here for a stated reason.

- **Compile the graph.** A `tiler-compiler` dev-dependency on `tiler-artifact` would make the preamble four lines. `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` (`crates/tiler-runtime/tests/identity_join/main.rs`) walks `Cargo.lock`, which merges normal and development edges per package, so that edge puts `tiler-compiler` in the consumer's closure and fails the test. Reproduce by adding the dev-dependency and running `cargo nextest run -p tiler-runtime`. ADR 0081 item 2 fixes the consumer closure at `[tiler-artifact]`, so the guard is asserting what it says.
- **Build a candidate index region per operation.** This is what `crate::program::tests` does, through `tiler_ir::index` alone, and it is what the unit tests need anyway because their provider-provenance and dual-output fixture graphs are ones governed compilation refuses. It runs to roughly 150 lines for the five-operation fixture graph, which is not a documentation example.

## Candidate resolutions, none chosen

1. Narrow the closure walk to dev edges of the *root* package. A dependency's dev-dependencies are not linked into a downstream crate, so the current walk over-approximates. This is a change to an accepted architectural guard and needs its own reasoning and Tom's view; it must not be made to fit a documentation example.
2. Shrink the example's graph to one operation whose candidate region is a few lines (an `F32Constant`), and show the receipt path in the open rather than hidden. This changes what the example demonstrates.
3. Accept the `ignore` and delete the preamble, showing the artifact builder against a program the text says is obtained elsewhere. Cheapest, and loses the end-to-end reading.

## Closes when

Both module examples compile in `cargo test --workspace --doc`, or the record says explicitly which of the above was chosen and why the loss is accepted.
