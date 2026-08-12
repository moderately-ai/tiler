---
id: refuse-two-structurally-identical-output-chains-by-name-not-as-compiler-output
title: Carry complete canonical ownership in every program stage
status: todo
priority: p2
dependencies: [reproduce-the-identical-output-chain-stage-key-collision]
related: [bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals]
scopes: [implementation/compiler, implementation/ir, implementation/artifact, implementation/build, implementation/runtime, contracts/foundation, contracts/artifacts]
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
