---
id: assemble-a-kernel-program-from-an-arbitrary-cover
title: Assemble a kernel program from an arbitrary cover
status: todo
priority: p1
dependencies: [derive-physical-proposals-from-the-cover-region-subject]
related: [define-the-minimum-correct-physical-realization-profile, implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, activate-shared-work-duplication-on-the-compile-path, widen-the-deterministic-budgets-to-the-decoder-layer-program]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, program-assembly, baseline]
---
## User-visible outcome

A retained plan over a cover of any region count assembles into a verified kernel program, with stages, buffers, allocations, data dependencies, and named outputs derived from the cover and its materialization edges. A cover the assembler cannot express reaches a caller as a typed refusal naming the region, not as invalid compiler output.

## Why this exists

This is obligation 3 of the [minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md), and its current failure class is the profile's sharpest defect: a deterministic topological partition of any program with more than three occurrences produces a plan shape the assembler classifies as a **compiler fault**. The first thing a correct baseline does would look like a bug in the compiler.

**Fact — assembly matches exactly three plan shapes, verified by reading at `57474a09`.** `build_plan_program` in `crates/tiler-compiler/src/pipeline/planning.rs` matches `(kind, scheduled)` against `(Fused, [region])`, `(Materialized, [_, _])`, and `(Materialized, [_, _, _])`, and returns `ProgramError::Structure { rule: "unsupported-plan-shape" }` otherwise. `verify_artifact_refinements` in `crates/tiler-compiler/src/program.rs` carries the same three-way match over `scheduled` with `rule: "artifact-strategy-cardinality"`, immediately above the `output_count() != 1` guard under `rule: "semantic-output-coverage"`.

**Fact — the assemblers read the recognizer's subject, not the cover.** `build_materialized_core` and `build_split_core` (`crates/tiler-compiler/src/program.rs`) open with `let subject = request.serial_sum();` and take from it the input keys and their count, the input shape and element count, the output key, shape, and element count, and the member partitions each stage is recorded as covering. Buffers are minted positionally against that subject: one external allocation per declared input, one program allocation for the temporary, one for the output, plus one for the staged partials in the split case. Nothing consults the cover's `MaterializationEdge` list, which already carries the value, producer position, result position, element count, producer occurrence, and consumer occurrences that a general assembler needs.

**Fact — `tiler-ir` is not the wall, and this was verified rather than assumed.** [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) located it: `KernelProgramBuilder::push_output` is general and bounded by `MAX_PROGRAM_OUTPUTS` (4096) rather than by one, `program::tests::storage_reuse_is_admitted_only_with_an_explicit_handoff` builds and verifies a two-output four-stage program over five allocations, and `ValueRole::fills` states that "a stage binds its buffers to values positionally, so the ordinal is discharged by that position".

## Why this is a dependency rather than a parallel track

**Inference, and it is falsifiable.** Program assembly is reached only for a *retained plan*, and a plan is retained only when every one of its cover's regions has an admitted implementation. Before [`derive-physical-proposals-from-the-cover-region-subject`](derive-physical-proposals-from-the-cover-region-subject.md) lands, no cover containing a region outside the three recognized member sets ever produces a plan, so a generalized assembler's new paths would be unreachable from the compile path and exercisable only through a fixture constructing a plan the compiler cannot produce — a check that cannot fail on the real path, which is not evidence. If a compile-path route that retains a four-region plan without a generalized provider is demonstrated, this ordering is refuted and the dependency should be dropped.

## Correctness argument

The generalization changes where each structural quantity is *read from*; it proves nothing itself, and the authority that proves things is unchanged.

1. **`KernelProgramBuilder::build` remains the whole-program authority.** Complete disjoint coverage of the semantic graph, a unique writer per materialized value, deliberate duplication of pure work, boundary-contract satisfaction, temporary initialization and lifetimes, aliasing, ordered opaque effects, ABI and launch references, and named-output coverage are all proven there. The generalized assembler's correctness argument is that it constructs the same obligations for N regions that the current one constructs for two — one value and one allocation per materialization edge, one stage per region, one data dependency per edge.
2. **The cover is already a verified authority for exactly these facts.** `verify_cover` re-derives it from the program and checks each ordered named output is produced by exactly one region rather than merely retained by some region. Deriving assembly from it is reading a checked value, not trusting an unchecked one.
3. **Reclassifying the failure is a correction, not a relaxation.** A cover the assembler cannot express is a coverage gap, which is a *missing compilation capability* under [the optimizer contract](../docs/compiler/optimizer.md#compilation-boundary-and-failure-classes)'s five classes. Reporting it as invalid compiler output claims the compiler produced something wrong when it produced nothing at all, and the two classes are not interchangeable: one tells a caller their installed authority is incomplete, the other says the compiler has a bug.
4. **Existing programs must not move.** The one-, two-, and three-region shapes must assemble to byte-identical programs and artifact plans, because those bytes are artifact identity.

## What must be true when this lands

1. **Every structural quantity comes from the cover or the semantic program.** Program inputs and keys from the semantic program's declared inputs; one internal value and one program-owned allocation per materialization edge, sized by the edge's own element count; one output value per ordered named program output, in declaration order; one stage per scheduled region, ordered so producers precede consumers; one data dependency per edge, from the producing stage to each consuming stage.
2. **Stage coverage is the cover region's members**, with the one documented exception that a subprogram's final pass covers nothing because the partial pass already claims the occurrence the two realize. That exception is declared through `push_partial_reduction`, which is what makes `UncoveringStage` admit it.
3. **An unassemblable cover is a typed refusal** naming the region and the reason. `"unsupported-plan-shape"` and `"artifact-strategy-cardinality"` are retired or reclassified out of the invalid-compiler-output class.
4. **`verify_artifact_refinements` re-derives by the same route**, so the build path and the receipt path do not acquire two independently maintained descriptions of what a cover assembles into — the duplicate-derivation hazard the three-way match already embodies twice.
5. **The `output_count() != 1` guard is untouched.** It is a separate obligation with a separate owner; relaxing it here without the request boundary's would admit a program the other refuses.
6. **The one-, two-, and three-region shapes are byte-identical** to their pre-change programs and artifact plans.

## Required failure-path evidence

Each run against a case that must fail, observed failing before it is trusted:

- A four-region cover assembles, and the same cover reports `unsupported-plan-shape` at the unchanged base — the perturbation that proves the new path is what admitted it.
- A cover whose materialization edge names an element count disagreeing with the producing region's write extent is refused, not silently resized.
- A plan naming fewer outputs than the program declares is refused by `KernelProgramDiagnostic::MissingNamedOutput` reached through the generalized route, not only through `tiler-ir`'s own test.
- A stage ordered before its producer is refused by the dependency check rather than assembled.
- The one-, two-, and three-region artifact plans are unchanged; perturb one allocation size and watch the assertion fail.

## Boundaries

- **Do not relax `output_count() != 1`.** [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) owns both guards and now depends on this ticket; that ticket's closing condition 2 previously absorbed this work and has been corrected to depend on it instead.
- **Do not widen the deterministic budgets.** A general cover's region, stage, and buffer counts are bounded by [`widen-the-deterministic-budgets-to-the-decoder-layer-program`](widen-the-deterministic-budgets-to-the-decoder-layer-program.md)'s subject; report the counts this change makes reachable as an input to it rather than moving a limit here.
- **Do not touch enumeration, dominance, retention, or plan selection.** This is an assembler.
- `contracts/optimizer` is declared because [the optimizer contract](../docs/compiler/optimizer.md#what-each-stage-is-general-over-today)'s stage-11 paragraph states the three-shape limit as a fact and becomes false in the same change.

## Stop conditions

- The generalization requires a `tiler-ir` widening. The evidence above says it does not; if it does, that widening is its own ticket and this one depends on it rather than widening a compiler path onto a physical layer that cannot express the result.
- The generalization forces an artifact-identity or program-domain version step. Draft it, file the carrier, and stop.

## Graph maintenance

- [`activate-shared-work-duplication-on-the-compile-path`](activate-shared-work-duplication-on-the-compile-path.md)'s activation trigger 2 — "`build_plan_program` assembles a kernel program from an arbitrary cover shape rather than three enumerated ones" — fires here; trigger 1 fires on this ticket's dependency. Both must fire before it leaves `deferred`. That ticket's triggers do not yet name these two tickets; the naming edit was inadmissible when these were filed because a live branch held a committed edit to the same file.
- Correct the optimizer contract's stage-11 paragraph and the [general compilation boundary](../docs/research/program-planning/general-compilation-boundary.md#the-critical-path-to-a-naive-but-general-compiled-mimo-program)'s item 2 in the same change.
- When this lands, re-read [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md)'s closing condition 2 rather than assuming it is discharged: this ticket removes the shape match, and that ticket still owns both output-arity guards.

## Closes when

1. A retained plan over a cover of any region count assembles, with every structural quantity derived from the cover or the semantic program.
2. An unassemblable cover is a typed missing-capability refusal naming the region, and no longer invalid compiler output.
3. `verify_artifact_refinements` re-derives through the same route as the build path.
4. The one-, two-, and three-region shapes are byte-identical, and each new check has been observed failing.
5. The contract sentences that stated the three-shape limit are corrected in the same change.
