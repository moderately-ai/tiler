---
id: carry-symbolic-extents-into-the-semantic-program
title: Carry symbolic extents from an inline region into the semantic program
status: todo
priority: p1
dependencies: []
related: [prototype-inline-proc-macro-frontend, promote-the-symbolic-index-profile-to-a-public-boundary, prototype-inline-aot-integration-proof]
scopes: [contracts/foundation, contracts/navigation, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [research, frontend, shapes]
---
## Why this exists

`prototype-inline-proc-macro-frontend` delivered the approved `tiler::tensor!` region and found that its central feature cannot reach the compiler at all.

**Fact.** `tiler_ir::shape::Shape` is a fixed-extent vocabulary — `crates/tiler-ir/src/shape.rs:1` calls itself "Target-independent **fixed** shape vocabulary" and `Extent` wraps a `u64`. `SemanticProgramBuilder::input` takes a `Shape`, so a semantic program's operand shapes are concrete numbers. Reproduce with `grep -n "pub struct Extent" crates/tiler-ir/src/shape.rs`.

**Fact.** A region's `sym n` binds at `AvailabilityPhase::LiveDevicePreflight` with `FactProvenance::RuntimeValidated` (`crates/tiler-macros/src/binding.rs`), which is to say from the values the invocation is handed, at run time. There is no extent at expansion time.

**Inference.** An inline region carrying a symbolic extent therefore cannot be constructed as a `SemanticProgram` while it is being expanded, and so cannot be verified, normalized, optimized, scheduled, lowered, compiled, or AOT-delivered. `crates/tiler-macros/src/region.rs` records this as `ProgramEvidence::DeferredSymbolicExtent` and refuses to substitute a representative extent, because a program built over invented extents would be a different program and its identity would name something no consumer wrote.

Symbols do reach the *index* layer — `SourcedExtent::Symbol`, `IndexRegionBuilder::new_with_shape_environment`, `sourced_tensor` — so the gap is specifically between the region text and the semantic program, not the absence of a symbolic vocabulary. `docs/ir.md` already states the boundary: "Completing this bounded static-extent profile will not complete the symbolic contract above."

## User-visible outcome

An inline region declaring `sym n` reaches the same compiler path a fully literal region reaches, so the accepted inline AOT flow is available to the syntax Tom approved rather than only to its fully specialized subset.

## What this must decide

- Whether a symbolic semantic shape is a widening of `Shape`, a distinct sourced shape at the semantic layer mirroring `SourcedShape`, or a specialization step that fixes extents from a caller-supplied environment before the semantic program is built.
- How `ShapeEnvIdentity` participates in semantic and artifact identity, so two regions declaring one interface remain one subject.
- What a frontend does when an extent is genuinely unknown until dispatch: specialize per observed extent and cache, or carry the symbol through to a guarded plan.

Each of those changes a public boundary, so this is a research ticket before it is an implementation one.

## Do not

Do not close this by having the frontend invent extents, by compiling a representative specialization and reusing its artifact, or by moving program construction into generated runtime code — the last is the runtime JIT the accepted inline developer experience forbids outright.

## Outcome

**All three questions are decided by elimination, and each has exactly one survivor.** The design record is [`docs/research/shapes/symbolic-semantic-extents.md`](../docs/research/shapes/symbolic-semantic-extents.md), written against base `bc39282`. Nothing here is self-accepted: seven atomic decisions are enumerated for Tom and seven delivery tickets are filed.

**Scope correction, disclosed rather than absorbed.** The ticket declared `contracts/foundation` alone. A design record under `docs/research/shapes/` needs `research/shapes`, and its catalog line lives in `docs/research/README.md`, which is `contracts/navigation` — the same pair `scope-the-sequence-extending-tensor-family` declared for the same reason. Both are added above.

**The gap is a vocabulary gap, not an availability gap — which is the finding that made every elimination cheap.** The expansion already builds a verified `ShapeEnv` (`crates/tiler-macros/src/binding.rs:466`): declarations, root bindings over `BindingSource::InputDimension` at `LiveDevicePreflight`, provenance, and a `ShapeEnvIdentity`. What it lacks is not information about `n` but a type that can carry a symbol into `SemanticProgramBuilder::input`. The ticket's framing that a symbolic region "cannot be constructed as a `SemanticProgram` while it is being expanded" is true of the current vocabulary and false of the situation: only the *model* is deferred to run time, never the environment.

**Question 1 — how a symbolic semantic shape is spelled. Survivor: relocate the promoted sourced vocabulary to `tiler_ir::shape` and have both layers consume it.** Widening `Shape` fails on four independent grounds, of which the sharpest is that `SourcedShape::Static(Shape)`'s normalization invariant becomes unstatable if a `Shape` may itself hold a symbol — it breaks a boundary Tom accepted on 2026-07-31. A semantic-layer *mirror* fails on the one-vocabulary key from that same acceptance: `SourcedExtent` is documented as "the crate's *one* constant-or-symbol vocabulary" precisely to prevent a second enum with a second encoder folding `ShapeEnvIdentity` into identity with nothing checking the two agree. Specialize-before-build fails on three grounds, and two of them are this ticket's own Do-nots reached by construction: the extent exists only at `LiveDevicePreflight`, so every route to a value is invent-it or build-the-program-at-run-time. The fourth candidate is not in the ticket and had to be named, because eliminating the three as written would have left the survivor set empty.

**Question 2 — `ShapeEnvIdentity` in semantic and artifact identity. Survivor: a fifth `SemanticIdentity` subject, folded once, with no downstream change at all.** Folding it into `SemanticGraphIdentity` is eliminated: `ShapeEnvIdentity` bundles declarations, root-binding provenance, and constraints, and the accepted three-identity table puts binding provenance on the *interface* side, while `SemanticGraphIdentity` is documented to identify graph meaning. Under the rejected candidate, two programs of identical meaning sourcing `n` from input `a` rather than `b` would report different graph identity. The composition analysis is that nothing downstream needs to change: `grep -rn "ShapeEnv" crates/tiler-artifact/src crates/tiler-cache/src` returns nothing (control: the same pattern over `crates/` returns five crates), and because the semantic subjects already travel inside `ComposedSubject`'s `ArtifactProgram` facet, the environment reaches the cache key with no new facet, no new dependency, and no crate learning what a shape environment is. A third facet would be the second-authority failure `compose-the-complete-expansion-cache-subject` eliminated. The ticket's "two regions declaring one interface remain one subject" requirement turns out to be closed *upstream* already, by the frontend's constant `REGION_SCOPE` and its order-independent canonical binding source. Residue, deferred with a trigger: a fifth subject does not split the environment's own three parts, and splitting reopens `tiler.shape-env.v3`.

**Question 3 — an extent unknown until dispatch. Survivor: carry the symbol to a guarded plan; the frontend does nothing special.** Per-extent specialization fails on four grounds. For the inline path the first is structural rather than economic: an observed extent exists only in the running consumer, and there is no compiler in a consumer's target graph, so "specialize and cache" is the runtime source JIT the accepted DX forbids outright. L5 supplies the fresh half — the runtime pipeline cache keys on specialization values, so specializing on `S` would mint a cold pipeline per decode step and put a mutable inference quantity in a cache key, which L5 already states as an owed artifact-assembly refusal.

**The finding that was not anticipated, and it removes a whole obligation.** The expected cost of carrying symbols through was that an unbounded extent yields an unprovable region, so `tensor!`'s `sym n;` would need interval syntax — a change to the grammar Tom approved. It does not, for the region shape this ticket is about. `a_wholly_undetermined_dynamic_copy_verifies_by_proved_extent_equality` builds a `[n] -> [n]` copy over an environment stating only `m == n`, with no interval anywhere, and it verifies with `BoundsProofView::ProvedExtentEquality` on every access and `CoordinatePermutation` ownership on the write; `ShapeEnv::proves_equal` is reflexive because `same_class` compares union-find roots. The approved elementwise region is exactly that same-symbol case. **Fact —** the inline frontend states no constraint today at all (`grep -n '\.require(\|\.guard(\|SemanticInputConstraint\|ExtentRelation' crates/tiler-macros/src/binding.rs` returns nothing), and for this profile that is correct rather than a gap. A bound is owed only where a symbolic extent must be related to a *different* extent, which is L5's case and `admit-an-additive-extent-relation`'s.

**Seven atomic decisions enumerated for Tom** in the record's own section: A1 relocation and the fate of the accepted index paths; A2 builder attachment with no setter; A3 `SemanticProgram::shape` returning the total view; A4 the fifth subject and whether it is optional; A5 `tiler.semantic-graph.v2` to `v3` with a tagged extent encoding while `tiler.shape-env.v3` stays; A6 the inference contract routing through `proves_equal` and the typed refusal a not-proved pair returns; A7 whether the `deliver` gate lifts on the strength of the no-interval finding.

**Deliberately not done.** No code changed and no type widened — this is the research ticket the three public boundaries required before implementation. `docs/ir.md` gained one paragraph stating the gap as a fact and pointing at the record; no other contract sentence moved, because none was false. The additive extent relation is untouched: it is already filed from L5 and the chain here neither depends on it nor duplicates it. No measurement was taken and none is claimed; the C1 artifact-count arithmetic is arithmetic over L5's stated row.

**Verification.** `tkt lint` clean; `git diff --check` clean; `make full` green on the final commit — 1,751 workspace tests passed with 4 skipped, 610 release-profile numerical tests passed with 1 skipped, doc-tests, rustdoc, `ticketsplease lint`, and shellcheck all clean. The record's six reproducible checks each carry a positive control, and each check and each control was run.

**Guard verdict: WARN at exit 0, which is a coordination note and not a scope escape.** Eleven files changed, affected scopes exactly equal declared scopes. `tkt guard` reports a *direct* collision with `implement-the-typed-accuracy-contract-vocabulary`, which is in-progress and also declares `contracts/navigation`, and shared `project/tickets` collisions with two more. The whole navigation footprint here is one inserted line in `docs/research/README.md` — `git diff --stat bc39282 -- docs/research/README.md` reports `1 insertion(+)` — placed at its alphabetical position in a sorted catalog, so the integrator should expect a one-line textual conflict there and nothing structural.

**Environment note, because it cost a run and could cost another.** The first `make full` failed with `ENOSPC`: the root filesystem reached 100% with 118 MiB free while a second agent ran `make full` in a parallel worktree. Only this ticket's own regenerable `target/` was removed — other worktrees' build outputs belong to their tickets and were left alone — after which the gate ran clean. Four worktree target directories held about 12 GiB between them at the time.

## Graph maintenance

- Seven delivery tickets filed, dependency-ordered: `relocate-the-sourced-extent-vocabulary-to-the-shape-module`, `carry-a-sourced-shape-on-semantic-values`, `resolve-semantic-shape-inference-over-symbolic-extents`, `fold-the-shape-environment-into-semantic-identity`, `construct-a-symbolic-region-as-a-semantic-program`, `admit-symbolic-extents-at-the-compiler-request-boundary`, `deliver-an-artifact-family-from-a-symbolic-region`.
- This ticket's stated outcome was a decision, and the decision is made, so it closes rather than holding in review — the delivery is the chain above and `review` does not satisfy dependents.
- `admit-an-additive-extent-relation` is unaffected and stays where L5 filed it.
