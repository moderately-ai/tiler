---
id: resolve-semantic-shape-inference-over-symbolic-extents
title: Resolve semantic shape inference over symbolic extents
status: awaiting-decision
priority: p1
dependencies: [carry-a-sourced-shape-on-semantic-values]
related: [carry-symbolic-extents-into-the-semantic-program]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, semantic-graph, decision, needs-tom, public-boundary]
---
## User-visible outcome

The registry decides whether two symbolic operands have one shape by asking the environment, so `f32[n] * f32[n]` is admitted for the right reason and `f32[n] * f32[m]` is refused with a typed reason naming both extents.

## Why this exists

**Fact — verified, with two repairs.** The governed profile's elementwise rule is "operand shapes must match or one operand must be scalar", and the frontend quotes it back verbatim. With fixed extents, "match" is `==` on `u64`. With symbols it cannot be, because two occurrences of one symbol are equal and two different symbols are not provably anything.

*Repair 1 — the citation was stale and is replaced by an anchor.* This Fact cited `crates/tiler-macros/src/region.rs:249`; at base `6eabf97e` that line is inside `AmbiguousReducedAxis`'s doc comment ("is *used* rather than where it is declared"). The sentence is in that file's module doc and in the `IncompatibleOperandShapes` `Display` arm — search `shapes must match or one operand must be scalar`. The same stale number appears in [the record this ticket was filed from](../docs/research/shapes/symbolic-semantic-extents.md), which this ticket cannot edit (scope `research/shapes`); it is reported for a worker holding that scope.

*Repair 2 — "the exact sentence" implied one authority and there are two.* The registry states it in **both** `BinaryF32` (`crates/tiler-ir/src/semantic/registry.rs`) and `BinaryBf16` (`crates/tiler-ir/src/semantic/bf16.rs`); search `operand shapes must match` for the two sites. A worker following this ticket literally would have widened one and left the other, and the two would then disagree about what a shape is. Both now resolve through one shared body, `elementwise_binary_shape`.

**Fact — verified in full at base `6eabf97e`.** `ExtentSources::proves_equal` is the accepted answer and it is deliberately one-sided: "`true` is a proof of equality; `false` means *not proved*, never *proved different*." It reaches `true` by an equality class or by a common determined value, and `ShapeEnv::proves_equal` is reflexive because `same_class` compares union-find roots. Read at `crates/tiler-ir/src/shape/sourced.rs` (`fn proves_equal`, and its "One-sided, and deliberately so" paragraph) and `crates/tiler-ir/src/shape/env/constraint.rs` (`fn same_class`, whose body is `self.classes.find(left) == self.classes.find(right)`).

## Implementation keys

- Route symbolic operand comparison through `ExtentSources::proves_equal` and nothing else. Do not add a syntactic symbol-identity shortcut beside it: the environment is the authority, and a second one would disagree the first time a constraint forces two differently spelled symbols together.
- A not-proved pair is a refusal, never a deferral and never a widening. It must reach the caller as a typed `BuildError` naming both extents, distinct from the shape mismatch, because a caller acts differently on "these are different sizes" and "this environment does not prove they are the same".

  **Corrected — this key said "distinct from the existing shape mismatch" and there is no such `BuildError` variant.** At base `6eabf97e`, `crates/tiler-ir/src/semantic/error.rs` was read in full: `BuildError` has no shape-mismatch variant at all. A shape mismatch is a *provider* diagnostic — code `binary.shape` — and reaches a caller as `BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication)`. **The requirement survives; only the thing it was contrasted with was wrong.**

  **Delivered against the letter, and the deviation is deliberate.** No new `BuildError` variant was added. The typed refusal is `ExtentSourceError::ExtentsNotProvedEqual(Box<ExtentDisagreement>)` carrying the axis and both extents, surfaced through the *existing* `BuildError::ExtentSource`. That type's own documentation already scopes it to a refusal by the program's environment — "does not prove what using it there requires" — and `DivisorNotProvedPositive` is the precedent for a not-proved refusal living there. A second `BuildError` variant would be a second spelling of one refusal, and the shape layer is where `proves_equal` lives. The discrimination the key asks for is delivered: `BuildError::ExtentSource(ExtentsNotProvedEqual { .. })` for a not-proved pair, `BuildError::SemanticRegistry(..)` with code `binary.shape` for two different sizes or two different ranks.

  **The environment is named by role rather than by identity**, matching every sibling variant's wording ("this program's shape environment"). A caller holding the refusal also still holds the builder, and therefore the environment itself.
- A rank mismatch stays a rank mismatch. Do not let the symbolic path report it under the new variant.
- Scalar broadcast is decided on rank, not on extents, so it is unchanged; state that explicitly rather than leaving a reader to infer that the rule was reviewed.
- Result shape derivation must produce a `SourcedShape` that names the operand's symbol rather than a fresh one, so the result and its operands share an equality class by construction.

  **Under-specified, and resolved by preserving the pre-existing bias.** The result is the **left** operand's own boundary, which is exactly what the fixed rule did. This is observable where the two operands are proved equal but spelled differently: `f32[n] * f32[4]` under an environment determining `n == 4` yields `[n]`, while `f32[4] * f32[n]` yields `[4]`. Per-axis "prefer the symbol" was rejected — it would mint a boundary neither operand wrote, which is the "fresh one" this key forbids.

## Evidence

- `f32[n] * f32[n]` admitted, with the proof route asserted rather than only the outcome.
- `f32[n] * f32[m]` refused under an environment with no relation; the same pair admitted once the environment states `m == n`, so the acceptance is evidence about the environment rather than about the spelling.
- `f32[n] * f32[4]` refused when `n` is merely bounded and admitted when the environment determines `n == 4`.
- Each new check perturbed once and observed failing; the failure text is in the worker report.
- A rank disagreement over symbolic operands reported as `binary.shape` with no extent pair attached.
- Scalar broadcast against a symbolic operand admitted in both operand orders, with the result naming the symbol.
- A literal-only family declining a symbolic operand by name, beside its literal neighbour.

## Obligation this ticket omitted, discovered while working

`crates/tiler-ir/src/semantic/precondition.rs` carried an instruction addressed to this ticket. Its `static_subject_shape` helper `expect`ed every precondition subject to be literal, on the ground that "`SemanticProgramBuilder` refuses a symbolic operand before the operation exists", and its doc said outright: "Admitting symbolic operands must step this domain, and this is the site that says so." [`carry-a-sourced-shape-on-semantic-values`](carry-a-sourced-shape-on-semantic-values.md) assigned the step here. This ticket did not mention it, and leaving it would have left a **panic** on a path that is now reachable.

Discharged: the obligation encoder reads the subject through `SourcedShape::encode`, `static_subject_shape` is gone, and `OBLIGATION_DOMAIN` steps `tiler.semantic-precondition-obligation.v1` → `v2`.

## Public boundary

A labelled draft under ADR 0075; Tom accepts the exact included and excluded sets.

**Included — added.** `tiler_ir::shape::ExtentDisagreement` (struct, public fields `axis`/`left`/`right`); `ExtentSourceError::ExtentsNotProvedEqual(Box<ExtentDisagreement>)` and `ExtentSourceError::SymbolicExtentUnsupported { axis, symbol }`, with their rendered text; `SourcedShape::without_axes`; `impl From<Shape> for SourcedShape`; `impl Display for SourcedExtent` and `for SourcedShape`; `OperationInferenceRequest::extent_sources` and `::static_operand_shape`; `OperationInferenceError::from_extent_source` and `::extent_source`; `FrozenSemanticRegistry::infer_operation_with_extent_sources`.

**Included — changed.** `ValueFact::shape` returns `&SourcedShape` (was `&Shape`); `ValueFact::new` takes `impl Into<SourcedShape>` and is no longer `const`; `SemanticPreconditionDisproof::shape` returns `&SourcedShape`.

**Included — removed.** `BuildError::SymbolicOperandUnsupported`, superseded rather than deprecated (pre-production, no external consumers).

**Excluded — deliberately not added.** No `BuildError` variant; no `PartialEq<Shape> for SourcedShape`; no `OperationInferencer` trait method; no schema-level symbolic-operand declaration; no change to `FrozenSemanticRegistry::infer_operation`'s existing signature.

**Rendered text changed.** `ExtentSourceError`'s three pre-existing variants moved prefix `index-extent.` → `sourced-extent.` and "this region's shape environment" → "this program's". Both were stale after the vocabulary's relocation to `tiler_ir::shape`, and this ticket makes the semantic layer a live producer of them; nothing asserts the old strings.

## Decision packet — 2026-08-09

The implementation and its failure-path evidence are complete; only acceptance of the exact included, changed, removed, and excluded public surface above remains. **Recommendation: accept the draft as built.** It keeps proof of symbolic equality in `ExtentSources`, carries a not-proved result through the existing typed shape-environment error boundary, and avoids creating a second `BuildError` spelling or a second equality authority. **Strongest counterpoint:** changing `ValueFact::shape` and the inference request surface makes sourced shape a durable public concept even though every non-elementwise standard family still refuses symbolic operands; Tom may prefer to accept only the error vocabulary and keep the wider inference surface private until a second family consumes it.

Tom's answer is acceptance or a precise revision of the enumerated surface; it does not reopen the already-tested left-operand result-shape rule or authorize the unsupported families below.

## Unsupported cases

- **Only the elementwise binary families decide the symbolic question.** `multiply-f32`, `add-f32` and their `bf16` siblings. Every other family in the standard profile — broadcast, concatenate, contraction, gather, reindex, slice, softmax, rms-norm, silu, the three strict-affine quantization keys, and the strict serial sum — declines a symbolic operand by name through `OperationInferenceRequest::static_operand_shape`, returning `ExtentSourceError::SymbolicExtentUnsupported`. Several are shape-*preserving* and would survive a symbolic operand on the shape rule alone; they still decline, because their normative definitions, reference evaluation, and numerical conformance are stated over fixed extents and admitting a boundary none of those can evaluate would move the refusal downstream to a layer with no name for it.
- **An out-of-crate provider that has not been taught the question compares `SourcedShape`s structurally.** `SourcedShape: PartialEq` is spelling equality, which is *sound but incomplete* against `proves_equal`: it admits two occurrences of one symbol (agreeing with reflexivity) and refuses everything else, including a pair the environment does prove equal. No production out-of-crate provider can receive a symbolic operand today: the frontend still defers symbolic regions, and the artifact builder refuses symbolic interface extents. Absolute "nothing outside `tiler-ir` constructs a symbolic program" is false — `tiler-artifact` tests build a symbolic `SemanticProgram` via `try_standard_with_shape_environment` + `input_sourced` to prove refusal — but that is fixture construction, not a taught production provider.
- **`OBLIGATION_DOMAIN`'s step is unguarded, and so is every other identity domain in the tree.** Reverting it to `v1` and running `cargo nextest run --workspace` is green: 3181 passed. No test in the repository pins `tiler.semantic-graph.v3` or `tiler.index-region.v11` either, so this is the existing convention rather than a gap this change opened. Introducing domain pins is a repository-wide convention decision and is left to Tom.

  **Correction — 2026-08-10.** The present-tense unguarded / unpinned claim above is **retired**. It was accurate as a landing-time measurement (workspace suite green after obligation `v2`→`v1`: 3181 passed, before any domain census). After this ticket landed, [`pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate`](pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate.md) (`status: done`) added `crates/tiler-ir/src/domains.rs`; `PINNED_IDENTITY_DOMAINS` now includes `tiler.semantic-precondition-obligation.v2\0`, `tiler.semantic-graph.v3\0`, and `tiler.index-region.v11\0`, and reverting any of them reddens the census. Introducing domain pins for that IR population is no longer open work left to Tom.

## Closes when

Tom accepts or revises the exact public boundary, the accepted inclusion and exclusion list is recorded with provenance, and any requested revision is implemented without weakening the sourced-equality authority or the typed refusal evidence.
