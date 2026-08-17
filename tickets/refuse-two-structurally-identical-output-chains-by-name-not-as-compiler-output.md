---
id: refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output
title: Carry complete canonical ownership in every program stage
status: done
priority: p2
dependencies: [reproduce-the-identical-output-chain-stage-key-collision]
related: [bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals]
scopes: [implementation/compiler, implementation/ir, implementation/artifact, implementation/build, implementation/runtime, contracts/foundation, contracts/artifacts, research/target-profiles, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, ir, artifact, identity-domain, multi-output]
---
## User-visible outcome

A valid program declaring two ordered named outputs whose producer chains are structurally identical compiles. Distinct continuation stages retain their exact semantic-realization ownership in canonical program and artifact stage identity, rather than colliding as `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))`, being merged despite distinct dataflow, or being refused by an arbitrary shape-sensitive support rule.

## Why this exists

**Historical Measurement, 2026-08-06, on `tkt/bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals` at base `afdac9c9`.** Two independent epilogue chains over two declared inputs — `sum(x * x, axis 1) * 2.0` published as `sx` and `sum(y * y, axis 1) * 3.0` published as `sy`, both at `[1, 4]` — failed `compile` with:

```
InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))
```

It was reproduced with the same fixture differing only in the prologue expression (`x * x` against `y + y`), and was *not* reproduced when the two chains folded different extents (`[1, 4]` and `[1, 2]`). The current compiler test source preserves that distinction under the source-verifiable anchor `identical shape assemble two stages carrying one canonical key`, but its executable fixture uses different extents and therefore does not re-prove the collision at this base. [`reproduce-the-identical-output-chain-stage-key-collision`](reproduce-the-identical-output-chain-stage-key-collision.md) now owns that current-boundary evidence.

**Inference — the public error class is wrong if the historical failure remains.** `AmbiguousCanonicalKey` is a `tiler_ir::program` core-verification refusal reported through `CompilerOutputError::Program`. The old measurement established that both outputs had passed recognition and assembly far enough to create the collision, but only the prerequisite can establish that current path. If it remains, either the program layer's stage key must distinguish the two stages, the compiler must refuse the request by name before assembly, or an actually equivalent stage must be shared.

## Prerequisite evidence — current at `b3b1652faa6c0060e4958782c2d5d37b563b9f8b`

**Measurement — 2026-08-10.** The prerequisite's permanent regression now re-proves the issue at public `session::compile`: two independent `[1, 4]` chains `sum(x * x, axis 1) * 2.0 -> sx` and `sum(y * y, axis 1) * 3.0 -> sy` under `REASSOCIATE_F32` return request-wide `Err`, class `InvalidCompilerOutput`, with a complete explain trace. A subject-only probe reports the private source as `InvalidCompilerOutput(Program(CoreVerification(AmbiguousCanonicalKey { entity: Stage })))`. The one-chain control and the same two-chain program with only `y` changed to `[1, 2]` compile.

**Fact — collision mechanism.** The equal keys are the two split combiners. Their bound kernel identities and launches agree; each physical pass owns a distinct fold occurrence's second semantic stage, but assembly projects only first-stage atoms into the shared program's coverage, leaving both combiner coverage lists empty. Outside the key, their internal bindings, views, values, allocations, split declarations, dependencies, and downstream `sx`/`sy` publications remain distinct. The current `stage_key` folds the agreeing kernel and empty coverage, and none of those distinguishing facts. The full commands, source anchors, exact comparison, and unsupported-case boundary are recorded in [`reproduce-the-identical-output-chain-stage-key-collision`](reproduce-the-identical-output-chain-stage-key-collision.md) under `Outcome — current evidence`.

## Decision boundary — accepted 2026-08-12

**Accepted by Tom at the live decision review on 2026-08-12, relayed through the coordinating agent.** Compile the valid program by making canonical stage ownership complete. Reject both alternative remedies: do not merge stages that dispatch equal kernel structure over distinct bindings, values, dependencies, reductions, and outputs; and do not introduce a request or recognition refusal whose support boundary depends on an otherwise irrelevant equality of extents.

The canonical owner is a required, closed two-way subject derived by provenance rather than by a numeric or structural heuristic:

- **realization-owned** — a nonempty canonical set of proof-bound `(occurrence, realization-stage ordinal)` claims; and
- **publication-owned** — a nonempty canonical set of exact `(output key, component role)` claims for an administrative publishing copy that computes no semantic occurrence.

There is no empty, unknown, optional, or default owner. A stage that computes or continues semantic work is realization-owned even when it also writes an interface result; publication ownership is the disjoint administrative-copy case. Future stage-accounting mechanisms must map explicitly into one of these exhaustive meanings or stop the owning matches at build time.

Preserve `CoveredOccurrence` as the stage-zero projection that discharges whole-program occurrence coverage exactly once. Add an exact proof-bound realization-stage claim for the richer per-stage subject; later split combiners and staged-family consumers retain their nonzero claims instead of projecting to empty identity. Add the occurrence to `PartialReduction`, because a producer may cover several fused occurrences and the verifier must not guess which one its combiner continues. Cross-check every nonzero claim against the exact split or staged-realization chain. Derive publication ownership from the existing publishing-copy, named-output, value-component, and writer facts rather than accepting a second caller-stated output relation.

Keep `PartialReduction`, `PublishingCopy`, and `StagedRealization` as distinct obligation records. The canonical-owner sum is their stage-identity projection, not a merged validation vocabulary with optional fields. Refuse missing ownership, foreign proof evidence, duplicate or skipped stage ordinals, an occurrence/declaration mismatch, a publication with no exact output component, and any ambiguous mixture instead of normalizing or falling back.

This deliberately supersedes one premise of the earlier accepted boundary without discarding its rationale. Occurrence coverage remains occurrence-scoped and proof-bound; the new evidence is that it is not the complete identity of every physical stage realizing that occurrence. The compiler already treats `(member, stage)` as a real attribution atom and folds it into region content. Dropping the stage component at program assembly is now demonstrated non-injective on a reachable valid population.

### Identity and compatibility consequence

The stage-subject record itself changes, so step the independently compared domains `tiler.kernel-program.stage.v2` to `v3` and `tiler.artifact-program.stage.v3` to `v4`. The verified program's stage-record grammar changes too, so step `tiler.kernel-program.v11` to `v12`. The artifact domain and manifest schema do not step: both already length-frame the complete independently versioned stage key and kernel-program identity, and no artifact record changes width, position, count, or interpretation. Their identity values, envelope digests, expansion-cache subjects, and recorded pins move transitively and must be recomputed from the integrated tree. Semantic-graph, request, target-profile, schedule, and structured-kernel identities do not move.

Do not fold accesses, materialized-value keys, dependency keys, or allocation keys into the stage key: the current canonical layering is intentionally acyclic (`stage -> value definition -> view -> allocation`), and feeding those downstream keys back into the stage would introduce recursive canonicalization. Do not use builder insertion order as a discriminator. The semantic owner is the acyclic label the model was missing.

## Scope note

The original compiler/IR-only scopes were false for the accepted remedy. `tiler-artifact` independently derives, compares, serializes, and tests the stage subject; `tiler-build` carries identity pins and maps program stages into artifact entries; runtime resolves opaque stage subjects into execution and deferred-entry positions; and both IR and artifact contracts own the version ledger. The expanded scopes are read from `ticketsplease.toml` and name that complete construction-and-consumption population.

## Closes when

- the public same-shaped pair compiles, while its different-extent and one-chain controls remain successful;
- the shared-IR and independent artifact encoders fold the same complete stage-owner record and their stepped domains are pinned;
- two realization stages differing only in occurrence, realization-stage ordinal, or proof evidence have distinct keys, while transient builder order, accesses, allocation placement, and declaration order do not become identity;
- two otherwise identical publication-only stages are distinguished by exact output key and component role;
- wrong occurrence, foreign evidence, duplicate/skipped ordinal, missing declaration, missing publication, and ambiguous ownership perturbations each fail by a named typed rule;
- every affected program, artifact, envelope, cache, and build pin is recomputed and its ledger updated, while the manifest and artifact-domain non-step derivations are recorded; and
- no request refusal, stage merge, builder-position discriminator, recursive key dependency, default owner, or silent fallback is introduced.

## Implementation evidence — 2026-08-17

**Fact — repaired before amendment.** The first implementation selected a split
occurrence from `producer.semantic_members().first()`, which was not an
accepted authority for a fused producer. `split_continuation_occurrence` now
derives exactly one member only where the producer's `SemanticStage` has an
exact `next_stage()` atom in the combiner coverage, and `build_cover_core`
cross-checks the declared assembly member against that derivation before
projecting it through `OccurrenceLowering`. Missing or ambiguous continuity is
the typed compiler structure refusal; an asserted mismatched occurrence is
`assembly-split-occurrence-mismatch`.

**Fact — direct owner and identity evidence.**
`cargo test -p tiler-ir --lib complete_stage_owner_refusals_reach_their_exact_graph_branches`
exercises missing ownership, foreign root, fork, loop, merge/disconnected path,
missing publication, and mixed realization/publication subjects and observes
their named `KernelProgramDiagnostic` branches. `complete_stage_owner_identity_changes_only_for_admitted_owner_claims` independently changes occurrence, reached proof, and continuation ordinal and observes different owner bytes; its equal control has no downstream value, allocation, dependency, or builder-order input. `cargo test -p tiler-artifact --lib the_artifact_stage_key_encodes_the_complete_kernel_program_stage_subject`
reconstructs the complete v4 subject and compares its tag/count/claims against
both the independently serialized key and the kernel-program identity. Its two
controls are a realization owner and a closed administrative
intermediate-to-named-output copy, so the latter reaches the publication tag,
count, exact output key, and `None` component-role framing. The generation test
reconstructs v1, v2, v3, and live v4 separately.
`complete_stage_owner_identity_changes_only_for_admitted_owner_claims` also
constructs the crate-private `StageOwner::Publication` subject directly and
proves that its `published`/`None` baseline differs independently for a changed
key and for `Some(EncodedComponentRole::new(99))`. That is encoder evidence,
not a claim that the current verified producer can emit a nonempty component
publication.

**Fact — negative controls.** The compiler continuity test changes the combiner
subject to an unrelated `next_stage()` and receives
`SplitContinuationError::Missing`; the owner tests mutate the graph subjects,
not their expectations. Temporarily changing the production artifact
realization owner tag from `0x01` to `0x03` and running
`cargo test -p tiler-artifact --lib the_artifact_stage_key_encodes_the_complete_kernel_program_stage_subject`
fails at `crates/tiler-artifact/src/program/tests.rs` with
`assertion \`left == right\` failed`; the independently reconstructed subject
retains `0x01` while the emitted artifact key carries `0x03`. The probe was
restored before the gates below. The amended publication control separately
changes production publication tag `0x02 -> 0x03`, derives
`perturbed-publication-key` instead of the actual output key, and derives
`Some(EncodedComponentRole::new(99))` instead of the actual `None` role. Each
unchanged agreement assertion fails with `assertion \`left == right\` failed`;
all three subjects were restored before the gates.

**Fact — component-role measurement boundary.** The live control reaches the
`None` role's explicit framing but cannot claim a nonempty role payload at this
base. `KernelProgramBuilder::check_origin` admits `Some(EncodedComponentRole)`
only for a semantic encoded output component, `verify_components` requires the
entire declared component set, and internal temporary values reject a component
role. The only existing artifact component kernel,
`strict_affine_u4_dequantize_kernel`, produces a plain F32 output; no current
artifact fixture/kernel writes an encoded output component. Constructing that
population would require a new encoded-result semantic operation, its
refinement/lowering authority, and component-output kernel(s), so it is outside
this accepted ownership and identity repair. This ticket therefore records the
supported `None` framing probe rather than inventing nonempty-component support;
the crate-private owner subject test above supplies the narrower nonempty-role
encoding evidence without implying producer reachability.

## Independent review and integration — 2026-08-17

Independent exact-commit review of `e5f1720db47354b5a63e9f7b7e7d154e76d75661`
over `796f87e58c2eec6a1e8c813f98ec9c16c882ac54` found no remaining issue.
The review re-derived split continuity, the closed owner graph and refusal
precedence, both independent stage encoders, every identity-domain step and
non-step, and the public same-shaped-output regression. It independently
perturbed the publication tag, output key, and `None` role framing; each made
the unchanged complete-subject agreement fail before restoration. Seven focused
cross-crate checks passed, followed by 1,466 `tiler-ir`/`tiler-artifact` tests
with one skipped, warnings-denied Clippy and rustdoc, ticket lint, citations,
diff checking, and exact-base scope guard with no conflict or under-declaration.

The reviewed hash was integrated unchanged by merge commit
`6c55d77a619c9752e005abcc54c027e47021fcc7`. The coordinator re-read the
cumulative 23-file diff and the final publication-only amendments before the
merge. Repository-wide publication gates and the final pushed commit are
recorded in the closing commit that marks this ticket done.
