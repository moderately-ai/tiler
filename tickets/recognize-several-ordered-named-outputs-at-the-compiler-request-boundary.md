---
id: recognize-several-ordered-named-outputs-at-the-compiler-request-boundary
title: Recognize several ordered named outputs at the compiler request boundary
status: in-progress
priority: p1
dependencies: []
related: [admit-ordered-multi-output-programs-at-the-compiler-request-boundary, admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary, admit-elementwise-epilogues-over-a-materialized-intermediate, assemble-a-kernel-program-from-an-arbitrary-cover]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api]
claimed_from: todo
assignee: agent-recognizer-widen
lease_expires_at: 1785950919
---
## User-visible outcome

The compiler request boundary recognizes a program declaring several ordered named outputs, producing a normalized subject that names one implementable region partition per output rather than one whole-program strategy reached from a single output.

## Why this exists

**Fact — the recognition is single-output by construction, and it is the only remaining wall.** `select_supported_strategy` (`crates/tiler-compiler/src/request.rs`) reads `program.outputs().next()`, classifies that one occurrence as a serial sum, a contraction, or an elementwise expression, and each recognizer below it then requires its walk to cover the program *exactly* — `recognize_pointwise` refuses under `operation-set` when `recognized.members.len() != program.operation_count()`, and `recognize_reduction` does the same through `check_recognized_operation_cover`. The result is one `NormalizedProgram`: one expression, one output shape, one output key, and one member partition. `crate::physical::spell_region` can spell only a cover region whose members equal one part of that partition, and `pointwise_region` builds its region from the single recognized expression.

**Fact — every layer this used to be blamed on has been cleared.** [`implement-general-dag-partitioning`](implement-general-dag-partitioning.md) landed: `crates/tiler-compiler/src/cover.rs` collects the ordered named program outputs a cover must produce and `verify_cover` checks each is produced by exactly one region. [`assemble-a-kernel-program-from-an-arbitrary-cover`](assemble-a-kernel-program-from-an-arbitrary-cover.md) landed: the three fixed plan shapes are gone, replaced by `CoverAssembly::from_plan` over a cover of any region count. [`carry-artifact-program-output-order-into-kernel-program-identity`](carry-artifact-program-output-order-into-kernel-program-identity.md) landed: output order is in kernel-program identity. `tiler-ir` was never the wall — `KernelProgramBuilder::push_output` is bounded by `MAX_PROGRAM_OUTPUTS`, not by one.

**Inference — the admissible multi-output set is empty until this lands, so the `output-arity` guard must not move first.** A second declared output either has its producing occurrence inside the first output's recognition walk or it does not. Outside the walk, the walk covers less than the program and `operation-set` refuses. Inside the walk, that value is consumed by another occurrence of the same walk, so its producing region either materializes for a consumer in another region — leaving its one owning write to serve both the materialization edge and the publication — or contains that consumer and must publish two named outputs from one write. A region writes one owning tensor and `ValueRole` is exclusive, so both are refused a layer down.

**Measurement (2026-08-05, at `3adc0689`).** Both branches were observed rather than argued, by relaxing `select_supported_strategy`'s `output_count() != 1` and `verify_artifact_refinements`'s `semantic-output-coverage` arity check together and compiling two fixtures through the ordinary entry point. An independent two-output program (`doubled = x * 2`, `tripled = x * 3`, standard registry) reached `phase: "strategy", rule: "operation-set"`. The reduction-epilogue fixture from `pipeline::conformance` — which publishes `scaled` and reduces it into `reduced` — reached `phase: "program-assembly", rule: "cover-named-output-attribution"`. The perturbation was reverted; nothing of it is retained in the tree.

## Boundaries

- This ticket owns the *recognition* and the normalized subject it produces. It does not relax either `output_count() != 1` guard — [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md) owns that, and depends on this.
- The epilogue shape — a program that publishes an intermediate *and* consumes it — needs a copy stage reading `TensorRole::Intermediate` and writing `TensorRole::Output`, which [`admit-elementwise-epilogues-over-a-materialized-intermediate`](admit-elementwise-epilogues-over-a-materialized-intermediate.md) owns. Programs with *independent* outputs do not hit it, and are the reachable target here.
- **`NormalizedProgramSubject` is identity-bearing.** It sits inside `VerifiedRequestSubject`, which every `VerifiedScheduledRegion` and the `ArtifactConstructionPlan` carry. Widening it is an identity-domain step: the version moves at its owning layer, the ledger documents move in the same commit, and every pinned identity is recomputed on the tree the step lands into, with each moved pin enumerated. Do not half-execute it.
- The public boundary of whatever replaces `NormalizedProgram` is Tom's to accept, not a worker's to self-accept.

## Required failure-path evidence

Each observed failing against an accepted neighbour: a program declaring two outputs whose second producing occurrence lies outside the first's walk, once the walk is per-output, must compile rather than refuse under `operation-set`; a program with an occurrence covered by *no* output's walk must still refuse under `operation-set`, so the coverage check is widened rather than removed; two ordered outputs attributed to the same region must refuse rather than publish twice from one write; and two programs differing only in output order must keep distinct identities through the new subject.

## Closes when

1. The request boundary produces a normalized subject naming one implementable region partition per ordered named output, and `spell_region` selects per cover region against it rather than against one whole-program partition.
2. The whole-program operation-cover check is widened to "every occurrence is covered by some output's walk" rather than removed, with the removal-shaped failure observed.
3. An independent two-output program reaches cover enumeration and complete-plan selection — the guard relaxation itself remains [`admit-ordered-multi-output-programs-at-the-compiler-request-boundary`](admit-ordered-multi-output-programs-at-the-compiler-request-boundary.md)'s.
4. If the normalized subject's encoding changes, the identity-domain step is executed completely in one commit, with every moved pin enumerated in the report.
