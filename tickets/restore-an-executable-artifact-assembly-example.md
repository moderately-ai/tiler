---
id: restore-an-executable-artifact-assembly-example
title: Restore the three assembly examples proof-bound coverage made unbuildable
status: done
priority: p3
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: [project/tickets]
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

Reproduce the population with `grep -rn '```ignore' crates/` at the commit that introduced them, which returned these three plus one pre-existing unrelated case in `crates/tiler-ir/src/index/builder.rs`.

**Correction — 2026-08-10.** Line numbers in the population table (`:164`, `:83`, `:104`) and that commit-time `grep` inventory were as of the coverage-binding commit that introduced the ignores, not the current base. At this base all three walk-through fences are ordinary open ```` ``` ```` (not ignore); the only remaining ```` ```ignore ```` in `crates/` is the unrelated Ordinary transactional call site illustration in `crates/tiler-ir/src/index/builder.rs`.

**All three are now pseudo-code, which is the sharper half of the problem.** Each calls a helper that exists nowhere — `refined_coverage()` in the `tiler-ir` example, `proof_derived_coverage()` in both `tiler-artifact` examples — standing in for the receipts the example cannot mint. An `ignore`d example that would compile if un-ignored is stale; one that names a function nobody wrote cannot be un-ignored at all without being rewritten first.

## Fact — why neither obvious route was taken

- **Compile the graph.** A `tiler-compiler` dev-dependency on `tiler-artifact` would make the two artifact preambles four lines each. `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider` (`crates/tiler-runtime/tests/identity_join/main.rs`) walks `Cargo.lock`, which merges normal and development edges per package, so that edge puts `tiler-compiler` in the consumer's closure and fails the test. Reproduce by adding the dev-dependency and running `cargo nextest run -p tiler-runtime`. ADR 0081 item 2 fixes the consumer closure at `[tiler-artifact]`, so the guard is asserting what it says. This route is unavailable to the `tiler-ir` example for a stronger reason: `tiler-compiler` depends on `tiler-ir`, so the edge is a cycle rather than a policy question.
- **Build a candidate index region per operation.** This is what `crate::program::tests` does in both crates, through `tiler_ir::index` alone, and it is what those suites need anyway because their provider-provenance and dual-output fixture graphs are ones governed compilation refuses. It runs to roughly 150 lines for the five-operation fixture graph, which is not a documentation example.

## Candidate resolutions

1. Narrow the closure walk to dev edges of the *root* package. A dependency's dev-dependencies are not linked into a downstream crate, so the current walk over-approximates. This is a change to an accepted architectural guard and needs its own reasoning and Tom's view; it must not be made to fit a documentation example. It also does not reach the `tiler-ir` example. **Not taken** — the guard was never touched, and `Cargo.lock` is unmodified.
2. Shrink each example's graph to one operation whose candidate region is a few lines, and show the receipt path in the open rather than hidden. **Taken, for all three**, with the operation chosen differently from the `F32Constant` this line originally named; the outcome below records why.
3. Demote the assembly preambles to prose: state that a verified kernel program is obtained from a lowering consumer, and show only the layer each module owns. Cheapest, loses the end-to-end reading, and is an honest outcome rather than a deferral. **Not taken** — none of the three needed it.

## Fact — outcome, per example

All three **compile** under `cargo test --workspace --doc`. Each mints its coverage from the sealed path a lowering consumer walks: derive the occurrence's `IndexRefinementSubject`, build a *candidate* index region, admit an `IndexRealizationAuthority`, and submit the pair to the refinement verifier, which mints the receipt only when the candidate's canonical identity equals the registered law's. No shortcut constructor was added and none was possible: `IndexRealizationLaw::realize` and `FrozenSemanticRegistry::index_realization_law` are both `pub(crate)`, so a doc-test — which compiles as its own crate, `tiler-ir`'s own included — cannot ask for the expected region and hand it straight back, and has to write the candidate out.

| Example | Verdict | What the reader sees |
| --- | --- | --- |
| `crates/tiler-ir/src/program/mod.rs` | compiles | receipt path and program assembly in the open |
| `crates/tiler-artifact/src/program/mod.rs` | compiles | receipt path and program assembly hidden; artifact packaging in the open |
| `crates/tiler-artifact/src/proof/mod.rs` | compiles | artifact assembly hidden; sidecar production and verification in the open |

**The one operation is an elementwise `F32Multiply` of two program inputs, not the `F32Constant` this ticket sketched, and the schedule verifier is why.** A single-stage kernel program needs a physical kernel family whose read binds a program *input*. `crates/tiler-ir/src/schedule/builder.rs`'s `verify_pointwise_f32` requires exactly one read access per expression input leaf, so a zero-read constant expression has no admissible access set. A bare `ScalarProgram::StrictSerialSum` is admissible as a single-stage program: serial (non multi-pass) `StrictSerialSum` uses `ContributorTensor::DeclaredDomain`, which admits `TensorRole::Intermediate` **or** the first input, so a single-region fold over a program input verifies. It was not chosen because a reduction's contributor map and axis/order topology are longer and less illustrative for a packaging walk-through than the shortest real single-occurrence program — `PointwiseF32` over two input leaves — which also demonstrates something a one-input sum would not: two input bindings, so the published interface order is observable. `FusedMultiplyAddSerialSum` does bind the first input, but it is the five-operation graph again.

**Correction — 2026-08-10.** An earlier form of this paragraph claimed bare `StrictSerialSum` is refused because `verify_access_and_semantics` requires `read.tensor == TensorRole::Intermediate` for that family, so a single-stage program would read an intermediate no stage writes. That Intermediate-only rule applies to multi-pass **Final** of a bare sum, not the serial single-region case. Live rule: `ContributorTensor::DeclaredDomain.admits` is `tensor == TensorRole::Intermediate || tensor == FIRST_INPUT` (`crates/tiler-ir/src/schedule/builder.rs`). The PointwiseF32 choice stands on the true reasons above, not Intermediate-only refusal.

The 2x3 subject and its 24-byte buffers are preserved, so the artifact and sidecar examples' visible text changed only where the interface widened from one input to two. The five-operation, multi-stage, partitioned-coverage case stays in `crate::program::tests` in both crates, and each module's prose now says so instead of apologizing for an `ignore`.

## Closes when

All three examples above either compile under `cargo test --workspace --doc` or are demoted to prose that names no function the workspace does not have — decided per example, with the choice and its reason recorded here. `grep -rn '```ignore' crates/` returns no site introduced by the coverage binding, and no example anywhere calls `refined_coverage` or `proof_derived_coverage`.

**Discharged.** `cargo test --workspace --doc` is green with all three examples running. `grep -rn "refined_coverage\|proof_derived_coverage" crates/` returns nothing. `grep -rn '```ignore' crates/` returns only the one pre-existing unrelated site — the Ordinary transactional call site illustration in `crates/tiler-ir/src/index/builder.rs` (unrelated to coverage binding). Each new doc-test was perturbed and watched to fail before being accepted, because a doc-test that silently stops running looks exactly like one that passes.

**Correction — 2026-08-10.** The discharged ignore citation previously named `builder.rs:1922`; that line number has drifted. Prefer the anchor "Ordinary transactional call site" over a line number. At this base the sole remaining ```` ```ignore ```` in `crates/` is that illustration.