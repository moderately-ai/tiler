---
id: assemble-a-kernel-program-from-an-arbitrary-cover
title: Assemble a kernel program from an arbitrary cover
status: done
priority: p1
dependencies: [derive-physical-proposals-from-the-cover-region-subject]
related: [define-the-minimum-correct-physical-realization-profile, implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, activate-shared-work-duplication-on-the-compile-path, widen-the-deterministic-budgets-to-the-decoder-layer-program]
scopes: [implementation/compiler, contracts/optimizer, research/program-planning]
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

## What landed, 2026-08-05

**The assembler is one route with one description.** `CoverAssembly` (`crates/tiler-compiler/src/program.rs`) is the whole structural description a retained plan's cover assembles into: the scheduled regions in execution order, one internal value per materialization edge and per staged subprogram pass and per ordered named output, one binding per stage access, the derived data dependencies, the declared splits, and the named outputs. `CoverAssembly::from_plan` is its only derivation on the compile path; `build_plan_program` and `verify_artifact_refinements` both consume it, so the build path and the receipt path no longer hold two descriptions of one thing. `build_materialized_core`, `build_split_core`, `build_fused_core`, and the three `build_*_kernel_program_with_lowering` entry points are gone rather than retained beside it.

**Stage order comes from the cover, not from a region identifier.** The retired `plan_region_order` sorted by `region().index.id`, which is a constant of the schedule vocabulary — every elementwise region carries `RegionId::new(0)` — so it returned an arbitrary order the moment a cover placed two regions one builder produced. The order is now a stable topological sort over the cover's materialization edges, tie-broken by the cover's own canonical occurrence order, so one cover has exactly one execution order.

**The reclassification landed with its explain consequence.** `"unsupported-plan-shape"` and `"artifact-strategy-cardinality"` are gone. An unassemblable cover reaches a caller as `CompileError::UnsupportedCapability(RequestError::UnsupportedCapability { phase: "program-assembly", rule })` — the missing-compilation-capability class — with the region occurrence label as the explain subject and the reason `unsupported-program-assembly-{rule}`, at `ExplainStage::ProgramVerification`. `"unlowerable-opaque-body"` deliberately keeps its class: lowering a body this compiler did not schedule is a separate capability with a separate owner, and moving it would be a second reclassification this ticket does not own.

**The `output_count() != 1` guard is untouched**, and the assembly derivation adds a *second* refusal beside it rather than a relaxation: the cover states which regions publish an output but not which named result each retains, so `cover-named-output-attribution` refuses more than one of either. That obligation is recorded on [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md), which owns supplying the attribution.

**Correction — 2026-08-10.** The landing-day sentence above is still true as a claim about *this* ticket: the assembler did not relax the arity guard. Present-tense "is untouched" is not live tree state. Later [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) removed both arity guards — `select_supported_strategy`'s `output_count() != 1` under `output-arity` and `verify_artifact_refinements`'s matching cardinality check under `semantic-output-coverage`. The live path compares ordered named-output keys for any `semantic.output_count()`; `cover-named-output-attribution` remains. Reproduce: `rg 'output_count\(\) != 1' crates/tiler-compiler` → history prose only.

**Byte-identity, checked rather than argued.** Every alternative's canonical kernel-program identity was printed before and after the change for the governed serial-sum compile and for the directly built fused, materialized, and split programs, and every one is byte-identical. One presentation-only quantity moves: the *declared* arena position of the applicability guard in the split (4 → 6), because the ABI byte expressions are now declared per value rather than in two passes. It is not identity-visible — `encode_identity` folds a canonical arena traversal from the use sites, and the artifact projects the guard through `expression_of[..]` — and the full workspace suite is green.

## The reachability premise this ticket was filed under is refuted

**Correction — 2026-08-10.** The Inference below that "no cover of more than two regions is retainable" and the claim that reachable programs are still only the one-region fused, two-region materialized, and three-stage split are **false as live present-tense Facts**. At this base three-region covers retain and assemble on the ordinary path (`outputs_reading_input_subsets_compile_and_bind_the_inputs_they_read` asserts `region_count() == 3` with three kernels); multi-output and published-and-consumed paths also assemble multi-region / multi-stage programs. The assembler remains general for any region count a retained plan can carry; reachable cover width grew with later vocabulary and recognition tickets. A four-region *compiled* program is still not a tested guarantee (no `region_count() == 4` assert found under `crates/`). The partition Fact understates the live binding vocabulary: `verify_region_subject_binding` also admits staged fold/pass, elementwise epilogue, publishing-copy, and single-workgroup tree arms, not only `normalized.members` / pointwise / reduction / all / empty combiner. Landing-day ordering rationale (assembler after provider) and the maturity claim (implemented support, not four-region tested guarantee) still stand.

**Fact, verified by reading `crates/tiler-compiler/src/physical.rs` in full.** `verify_region_subject_binding` admits a verified scheduled region only when its `semantic_members` equal one of the partitions the request-level recognizer pre-computed: `normalized.members` for a pointwise or contraction subject, and `members().pointwise()`, `members().reduction()`, or `members().all()` for a serial-sum subject — plus the empty set a split's combining pass claims. The exact check is `grep -n 'semantic_members ==' crates/tiler-compiler/src/physical.rs`, which returns one comparison per recognized partition and none against anything else.

**Inference — no cover of more than two regions is retainable, so no four-stage program is constructible anywhere.** The dependency's own landing commit says it plainly: "This widens nothing. The three walls still refuse the same programs." A cover placing a region outside those partitions has no proposal, so it retains no plan and never reaches assembly; and because `CoverAssembly` takes *verified* regions, the same binding stops a test from spelling a four-stage assembly by hand. The reachable programs are still the one-region fused, the two-region materialized, and the three-stage split.

**What this ticket's "Why this is a dependency rather than a parallel track" section got wrong.** The ordering was right — the assembler had to follow the provider — but its stated consequence, that landing the provider would make the new paths reachable, does not follow and is false. The correct statement is the one the profile records now: the walls were at stages 8 and 11, both are gone, and what bounds the compiled set is the **region vocabulary**, whose three widening tickets each convert a refusal into an offer with no further change at either stage.

**What is therefore claimed, in the four maturity classes AGENTS.md separates.** The N-region assembler is *implemented support* with its obligations proven by `KernelProgramBuilder::build`; it is **not** a *tested guarantee* over four-region programs, because none can be constructed. The evidence that exists: the derivation runs on the compile path for every compiled program; the general code paths for edges, staged passes, split declarations, execution ordering, and per-value dependencies are all exercised by the one-, two-, and three-stage programs; each constructed obligation is observed refusing when stated wrongly; and the ordering property is asserted over every cover the governed program enumerates.

## Closes when

1. A retained plan over a cover of any region count assembles, with every structural quantity derived from the cover or the semantic program.
2. An unassemblable cover is a typed missing-capability refusal naming the region, and no longer invalid compiler output.
3. `verify_artifact_refinements` re-derives through the same route as the build path.
4. The one-, two-, and three-region shapes are byte-identical, and each new check has been observed failing.
5. The contract sentences that stated the three-shape limit are corrected in the same change.

All five are satisfied as of 2026-08-05; the sections above record how, and record the one thing the closing conditions did not ask for and could not have been given — a compiled four-region program.

## Why `research/program-planning` is declared

Added 2026-08-05 by the implementing worker, as declaration and scheduling metadata rather than product scope. The ticket's own graph-maintenance section requires correcting [the general compilation boundary](../docs/research/program-planning/general-compilation-boundary.md#the-critical-path-to-a-naive-but-general-compiled-mimo-program)'s item 2, and closing the wall additionally makes the [minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md)'s wall-2 section, stage-11 table row, and assembly-specification status false in the same change. `ticketsplease.toml` maps both files to `research/program-planning`, and the ticket declared only `implementation/compiler` and `contracts/optimizer`. No other live ticket holds the scope — `tkt list --status in-progress` returned this ticket alone.
