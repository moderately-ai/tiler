---
id: restore-an-executable-artifact-assembly-example
title: Restore the three assembly examples proof-bound coverage made unbuildable
status: todo
priority: p3
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: []
paths: []
tags: [documentation, artifact]
---
## User-visible outcome

A reader following the module walk-throughs in `tiler_ir::program`, `tiler_artifact::program`, and `tiler_artifact::proof` is following code the gate compiles, rather than prose that may have drifted from the builders it demonstrates.

## Fact — what changed, and the exact population

`bind-stage-coverage-to-index-refinement-identity` made stage coverage proof-derived: `KernelProgramBuilder::push_stage` takes `CoveredOccurrence` records, and the only constructor for one takes a completed `tiler_ir::index::IndexRefinementReceipt`. Every example that assembles a kernel program therefore needs receipts, and **three** were marked ```` ```ignore ```` in that commit rather than left broken:

| Example | Site | Crate |
| --- | --- | --- |
| Kernel-program assembly | `crates/tiler-ir/src/program/mod.rs:164` | `tiler-ir` |
| Artifact assembly | `crates/tiler-artifact/src/program/mod.rs:83` | `tiler-artifact` |
| Proof sidecar beside an artifact | `crates/tiler-artifact/src/proof/mod.rs:104` | `tiler-artifact` |

Reproduce the population with `grep -rn '```ignore' crates/`, which returns these three plus one pre-existing unrelated case in `crates/tiler-ir/src/index/builder.rs`.

**All three are now pseudo-code, which is the sharper half of the problem.** Each calls a helper that exists nowhere — `refined_coverage()` in the `tiler-ir` example, `proof_derived_coverage()` in both `tiler-artifact` examples — standing in for the receipts the example cannot mint. An `ignore`d example that would compile if un-ignored is stale; one that names a function nobody wrote cannot be un-ignored at all without being rewritten first.

## Fact — why neither obvious route was taken

- **Compile the graph.** A `tiler-compiler` dev-dependency on `tiler-artifact` would make the two artifact preambles four lines each. `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` (`crates/tiler-runtime/tests/identity_join/main.rs`) walks `Cargo.lock`, which merges normal and development edges per package, so that edge puts `tiler-compiler` in the consumer's closure and fails the test. Reproduce by adding the dev-dependency and running `cargo nextest run -p tiler-runtime`. ADR 0081 item 2 fixes the consumer closure at `[tiler-artifact]`, so the guard is asserting what it says. This route is unavailable to the `tiler-ir` example for a stronger reason: `tiler-compiler` depends on `tiler-ir`, so the edge is a cycle rather than a policy question.
- **Build a candidate index region per operation.** This is what `crate::program::tests` does in both crates, through `tiler_ir::index` alone, and it is what those suites need anyway because their provider-provenance and dual-output fixture graphs are ones governed compilation refuses. It runs to roughly 150 lines for the five-operation fixture graph, which is not a documentation example.

## Candidate resolutions, none chosen

1. Narrow the closure walk to dev edges of the *root* package. A dependency's dev-dependencies are not linked into a downstream crate, so the current walk over-approximates. This is a change to an accepted architectural guard and needs its own reasoning and Tom's view; it must not be made to fit a documentation example. It also does not reach the `tiler-ir` example.
2. Shrink each example's graph to one operation whose candidate region is a few lines — an `F32Constant` — and show the receipt path in the open rather than hidden. This changes what the examples demonstrate, and is the only candidate that reaches all three.
3. Demote the assembly preambles to prose: state that a verified kernel program is obtained from a lowering consumer, and show only the layer each module owns. Cheapest, loses the end-to-end reading, and is an honest outcome rather than a deferral.

## Closes when

All three examples above either compile under `cargo test --workspace --doc` or are demoted to prose that names no function the workspace does not have — decided per example, with the choice and its reason recorded here. `grep -rn '```ignore' crates/` returns no site introduced by the coverage binding, and no example anywhere calls `refined_coverage` or `proof_derived_coverage`.
