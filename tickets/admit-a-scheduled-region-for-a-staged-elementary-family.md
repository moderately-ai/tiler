---
id: admit-a-scheduled-region-for-a-staged-elementary-family
title: Admit a scheduled region for a staged elementary family
status: review
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, accept-the-root-mean-square-scale-realization-law, accept-the-fold-with-epilogue-scheduled-region, accept-the-registered-family-realization-law-query, account-for-a-staged-realization-stage-in-the-kernel-program]
scopes: [implementation/ir, implementation/compiler, implementation/metal, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, identity-domain]
claimed_from: todo
assignee: agent-staged-region
lease_expires_at: 1786051056
---
## User-visible outcome

A program whose middle stage is a registered elementary family compiles end to end with reference bit-agreement, instead of stopping at `RegionVocabularyWall::StagedFamilyUnspellable`. This is the **physical half** [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) stopped at and filed; that ticket's recognizer half has landed, so every layer above this one already works.

## Where the wall is, verified on `b3d5a9ed` plus the recognizer landing

**Fact — everything above the scheduled region already runs.** For `rms_norm(value, weight) * value` over `[2, 2]`, `compile()` now: recognizes the normalization as `NormalizedOutput::Staged`; resolves `tiler.governed-index-access.rms-norm-f32@1`; refines it — `realization-stages: count=2`, one handed value, producer stage 0, consumer stage 1; enumerates one region candidate per stage; and enumerates four legal covers, including the one that fuses the normalization's pass beside its downstream consumer. `pipeline::tests::a_staged_family_program_reaches_its_lowering_and_names_the_vocabulary_wall` is the measurement.

**Fact — the missing thing is a `ScalarProgram`.** `tiler_ir::schedule::ScalarProgram` has eight variants (`crates/tiler-ir/src/schedule/model.rs:473`). Stage zero of `IndexRealizationLaw::StagedRootMeanSquareScaleF32` folds each contributor's *square* and then applies `/N`, `+eps`, and `Rsqrt` to the fold's value **inside the producing region**. `ScalarProgram::SquaredSerialSum` is the closest variant and carries no epilogue — deliberately, and its doc-comment says so: "the division by the extent, the `eps` addition, the reciprocal square root, and the two multiplies belong to the pointwise pass that consumes this reduction's result".

**Fact — that doc-comment and the accepted law disagree about where the epilogue lives**, and the disagreement is the substance of this ticket. The law's Outcome records why the epilogue is in stage zero: `r` is computed once per folded row and read once per point, so publishing `a` and putting `/N`, `+eps`, `Rsqrt` in the pointwise pass evaluates each `N` times per row — a different scalar program, not a different schedule. Refinement compares a provider's emission against the law byte for byte, so a scheduled region built the other way would not be the realization the compile path proved.

## The surface this touches

- **`tiler-ir` schedule vocabulary.** A `ScalarProgram` variant for a squared serial fold with a scalar epilogue on the fold's value, or a decision that the law moves instead. This is a public `#[non_exhaustive]` enum: land as a labelled draft with its own acceptance node, and derive the canonical region-identity encoding's per-tag injectivity at the encoding site.
- **`tiler-compiler` physical.** A `RegionSpellingKind` variant, a region builder, the `frontier::GovernedPhysicalProvider::propose` dispatch, `physical::spell_output`'s staged arm (which today returns the wall), and `physical::verify_region_output_binding`'s `(NormalizedOutputSubject::Staged, _)` arm (which today answers `false`). The recognized shape it binds against is already in place.
- **`tiler-compiler` program assembly.** A staged occurrence's handed value reaches `derive_materializations` as an ordinary `MaterializationEdge` today, but `CoverAssembly::from_plan` shapes each internal from the producing stage's `iteration_shape` and cross-checks it against the edge's `element_count` (`program.rs`, `materialized-extent-disagreement`). The normalization's handed value is one element per folded row while its fold stage iterates the *published* domain, so this is the first check to derive against rather than assume.
- **`tiler-metal` emission and `tiler-reference` evaluation** for the new scalar program, and the cost model.

## Non-goals

The softmax's law (blocked on its own two tickets); a staged family that *reads* a materialized intermediate ([`admit-a-staged-family-that-reads-a-materialized-intermediate`](admit-a-staged-family-that-reads-a-materialized-intermediate.md)); the failure-class question ([`classify-a-vocabulary-gap-refusal-as-an-unsupported-capability`](classify-a-vocabulary-gap-refusal-as-an-unsupported-capability.md)).

## Closes when

A program with a registered elementary family as a middle stage compiles through the ordinary path and agrees with `tiler-reference` bit for bit, the scheduled-region vocabulary addition is a labelled draft with an acceptance node, every moved identity pin is recomputed on the landing tree and enumerated, and the staged arm of `spell_output` returns a spelling instead of a wall.

## Outcome 2026-08-06 — the physical half landed whole; the last wall is program scope and is filed

Three of the four closing conditions hold. The fourth — *compiles through the ordinary path* — does not, and what stops it is one program-scope declaration whose identity step is not appends-only, which is a stop condition this ticket's dispatch named. The boundary is gated, typed, and named rather than silent.

### The law-vs-doc contradiction, resolved in the law's favour

`ScalarProgram::SquaredSerialSum`'s doc said the epilogue "belong[s] to the pointwise pass that consumes this reduction's result". The accepted law's Outcome records why it does not: `r` is computed once per folded row and read once per point, so the other placement evaluates it `N` times per row — a different scalar program, not a different schedule. Reading both in full found **no evidence the law's reasoning is wrong**, and one further fact against the doc: refinement compares a provider's emission against the law byte for byte, so a region built the other way would not realize the occurrence the compile path proved.

Resolved as the dispatch directed. The vocabulary gained what the law needs, and `SquaredSerialSum`'s paragraph is rewritten in place to current truth: the variant is the fold *alone*, and where a transform belongs is the operation's question rather than a schedule's. The variant itself, its tag, and its bytes are unchanged, and it remains correct for its own uses.

### What landed

**`ScalarProgram::SquaredSerialSumThenEpilogue`** (`crates/tiler-ir/src/schedule/model.rs`), a labelled draft with acceptance parked at [`accept-the-fold-with-epilogue-scheduled-region`](accept-the-fold-with-epilogue-scheduled-region.md). A squared serial fold whose value a **whole verified `PointwiseF32Expression`** then transforms, the expression's sole leaf being the fold's value rather than a boundary tensor.

*The generality argument, and the limitation named.* The **epilogue** is fully general: any chain the physical `f32` vocabulary spells is expressible without a further variant — a mean is `a / N`, this family's scale is `Rsqrt(a / N + eps)`, a reciprocal-sum normalizer is `c / a`. The **fold** stays one variant per (prologue, combiner) pair, which is the grain `SquaredSerialSum` and `StrictSerialMaximum` already set. So the softmax's shifting stage, which folds a *maximum*, will need its own sibling here rather than a field on this one; what it inherits unchanged is the epilogue field and every derivation threaded for it — the verifier's two rules, the identity payload, the lowering's epilogue hook, and the split refusal. The rejected alternative was a `fold: SerialFoldKind` field crossing three combiners with the epilogue: most of that product denotes programs nothing registers, and admitting unreachable combinations into an accepted vocabulary is what `law.rs`'s own header argues against. The trade is stated in the acceptance node as the thing to object to.

*Two rules the variant owes, both refusals.* An epilogue whose root is its own input leaf computes nothing and is a second spelling of `SquaredSerialSum` — refused, on the canonicality rule `broadcast_decodes_are_replicating` states for its own degenerate case. An epilogue naming a second input names a buffer no fold region binds — refused, so the lowering never reaches a handle error for it.

*No parallel topology may split it*, and the refusal is the algebra rather than caution: the epilogue applies to the *complete* fold, so a partial pass applying it transforms a fragment and one that does not is computing `SquaredSerialSum` under this variant's name. `multi_pass_family` and `cooperative_family` answer `None` and the topology is refused.

**Everything the variant reaches**: the identity encoder, the two total maps in `model.rs`, the intrinsic verifier's access-count group and its semantic arm, the kernel element type and signature arity, and the structured-kernel lowering — where `emit_reduction` gained an epilogue hook applied on *every* path out of the fold, including the empty-domain one, so the empty and singleton cases compute the same function as the general one.

**`FrozenIndexRealizationLawRegistry::family_realization_law`** (`crates/tiler-ir/src/index/refinement.rs`), a labelled draft with acceptance parked at [`accept-the-registered-family-realization-law-query`](accept-the-registered-family-realization-law-query.md). It answers the question the physical layer had no route to: *what does this stage compute*. Nothing else could serve — the shapes do not determine the axes (a `[2, 2]` operand handed a `[2]` value names two different reductions), and `resolve` needs a subject, which needs the whole `SemanticProgram`. The acceptance node names the live alternative for Tom: collapsing it with the parked `family_realizes_region_sequence`, which would need `IndexRealizationLaw::realizes_region_sequence` made public and would edit a node already parked for decision — which is why it was not taken during implementation.

**`physical::staged_plan`** (`crates/tiler-compiler/src/physical.rs`), the one place a law's meaning is translated into the scheduled vocabulary. **It is keyed on the law, never on the family**: the occurrence's operation key appears nowhere in it, so a second family registering `StagedRootMeanSquareScaleF32` is spelled by the same arm, and a family registered tomorrow with a law this profile has no arm for is refused by name under the fail-closed wildcard. Its refusals are the law's own, restated where a region would otherwise be built from facts the law would not have realized: a folded extent no binary32 value equals, an empty fold, a record that does not decode, disagreeing shapes. One refusal is this layer's rather than the law's and is named as such — `rms_norm(x, x)` reads one declared input twice densely, which `tiler_ir::schedule`'s read-ordering rule refuses as two spellings of one computation, so it is declined here rather than proposed and rejected.

**Two region spellings and two builders**: `RegionSpellingKind::StagedFold` (region 7) and `StagedPass` (region 8), the `spell_staged` arm that replaces the wall, the `GovernedPhysicalProvider::propose` dispatch with a cost row each, and the `verify_region_output_binding` staged arm — which re-derives the plan and compares the epilogue chain whole and the pass's read list tensor by tensor and relation by relation, fail-closed for every other pairing. The fold's spelling additionally requires a `Materialized` write role, because the value it writes is law-internal and no cover can publish it. `NormalizedStaged` gained the law and its typed attribute record; both are functions of what it already carried, so **no subject byte moves**.

*The consuming stage needed no new scalar program*: it is an ordinary `PointwiseF32` region whose read of the handed value is a `BroadcastReplication` at the kept coordinates, which the pointwise access contract already admits. What it needed was a builder derived from the law rather than from a recognized walk.

### Evidence

**Bit-for-bit agreement with `tiler-reference`.** `pipeline::tests::the_staged_regions_compute_the_normalization_bit_for_bit`: the two regions built by the compile path's own builders, each resubmitted through `verify_schedule` — intrinsic verification, numerical-realization comparison, request-subject binding, target feasibility — then lowered and interpreted in order over a fixture chosen so every rounding is observable. The result equals `ReferenceEvaluator::standard()`'s normalization bit for bit, against a reference that divides by the extent rather than by a reciprocal and certifies its reciprocal square root against an exact rational enclosure. Watched failing under a deliberate perturbation: associating the pass's two multiplies the other way — one function in exact arithmetic, two in binary32.

**The wall map flipped.** `a_staged_family_program_reaches_its_lowering_and_names_the_vocabulary_wall` is now `a_staged_family_program_spells_both_stages_and_names_the_program_scope_wall`, and its map moved from `{region-staged-family-unspellable: 3, region-partial-coverage: 2}` to `{region-staged-family-unspellable: 1, region-partial-coverage: 2}` — the one remaining staged wall is the region carrying *both* stages, which no scheduled region computes. Answered roles moved from `["epilogue"]` to `["epilogue", "staged-family", "staged-family"]`. Watched failing under a deliberate perturbation: reverting `spell_staged` to answer the wall for both stages.

**Seven new tests, each watched failing under a named perturbation**: the region verifies as a serial pass with two accesses; an identity epilogue is refused (perturbed by dropping the identity-root guard); a two-leaf epilogue is refused (perturbed by dropping the one-leaf rule); the epilogue payload separates canonical identity from the bare fold's and from another epilogue's (perturbed by encoding under `0x26` without the payload); no parallel topology admits it (perturbed by giving it the squaring fold's partial-pass admission); the epilogue is emitted once per output position rather than once per contributor (perturbed by dropping the epilogue argument to the lowering); and the two program-level tests above. `every_serial_fold_family_may_commit_to_a_materialized_intermediate` counts five fold arms rather than four.

**Identity: appends-only, zero moved pins.** Scalar-program tag `0x2A`; `0x22` through `0x29` keep their meanings and field positions; `tiler.schedule.v5` does not step. The per-tag injectivity is derived at the encoding site in both directions — every field reaches the bytes at a position the frames determine, and nothing but those fields does, the expression's own canonicalization giving one node order per meaning. `cargo nextest run --workspace` is **2878 passed, 7 skipped with no pin edited at all**.

**`tiler-metal` needed no arm.** It emits from the structured kernel rather than from `ScalarProgram`, and every operation the epilogue reaches — `F32Divide`, `F32Add`, `F32Multiply`, `F32Rsqrt`, `CanonicalizeF32Nan` — already has an emission. The cost row is `frontier.rs`'s structural estimate, which is keyed on `RegionSpellingKind`; both new kinds carry one, and the fold's launched-thread count is its *own* iteration count rather than the request's widest output, because a staged fold's domain is neither.

### Not landed, and why — the last wall is a program-scope declaration whose identity step is not appends-only

**Fact — the extent check the ticket flagged passes rather than refusing.** `materialized-extent-disagreement` was named as the first check to derive against rather than assume. Derived: the producing stage's iteration shape *is* the handed value's shape — one element per folded row, `[2]` for a `[2, 2]` occurrence reduced on axis one — and `graph.value_element_count` of the synthetic value is the same 2. Nothing needed weakening and nothing needed restating.

**Fact — the refusal is coverage.** `tiler_ir::program`'s `verify_partial_reductions` refuses a stage covering no occurrence under `UncoveringStage` unless a declaration accounts for it, and it admits exactly two: a declared split's combiner and a declared publishing copy's publisher. Coverage is keyed on `SemanticOccurrence` and refuses one occurrence twice, so `covered` projects a stage's atoms to its *first*-stage ones — a staged realization's later stage claims nothing at program scope.

**Fact — it fits neither account, and the arithmetic says so rather than the naming.** It is its own cover region, so it is neither pass of a split nor either dispatch of a copy; and it could not be *declared* as a split, because that contract requires `partial_elements == result_elements * partitions`, which here is `2 == 4 * partitions` and has no solution.

**Inference — a third declaration steps `tiler.kernel-program.v10`.** The `v10` step's own reasoning applies unchanged and is recorded at `model.rs:1643-1660`: a new declaration section is encoded unconditionally, so every program's bytes move, and the appended *conditional* alternative was considered and rejected there on grammar-determinacy grounds. Artifact identity, cache subjects, every pinned program identity, and an artifact-codec field move with it. That is an identity step that is not appends-only, which this ticket's dispatch named as a stop condition.

**What was done instead of forcing it.** The refusal is made typed and named at the layer that owns it: `CoverAssembly::from_plan` detects the unaccounted stage before the IR does and answers `AssemblyRefusal::missing(region, "realization-stage-unaccounted")`, whose `MissingCapability` class reaches a caller as `UnsupportedCapability { phase: "program-assembly", rule: "realization-stage-unaccounted" }` with the region named and the whole explain trace attached — a *missing compilation capability* rather than the invalid-compiler-output the IR would report for the same program. Filed with the full derivation and the enumerated surface as [`account-for-a-staged-realization-stage-in-the-kernel-program`](account-for-a-staged-realization-stage-in-the-kernel-program.md). Watched failing under a deliberate perturbation: dropping the refusal returns `InvalidCompilerOutput(Program(CoreVerification(UncoveringStage)))`.

### Unsupported cases, each refused by name

- A staged occurrence whose two operands are one declared input (`rms_norm(x, x)`) — declined at `staged_plan`, so the region is never proposed. A widening needs the read-ordering rule to admit two dense reads of one tensor, which is that rule's own decision.
- A folded extent no binary32 value equals, and an empty fold — the law's `rms-scale-extent-not-exact` and `rms-scale-empty-fold`, restated at `staged_plan` because it runs before any realization is built.
- A cover region carrying *both* stages — `region-staged-family-unspellable`, correctly: no scheduled region folds a contributor domain and evaluates a per-point expression over the fold's result, which is why the law realizes the occurrence as a sequence at all.
- A staged law this profile has no arm for — the fail-closed wildcard in `staged_plan`, reported as the same wall.
- A parallel split or cooperative tile of a fold carrying an epilogue — refused by the schedule verifier; the widening is a separate ticket rather than a relaxation.

### The support-matrix row this advances

`docs/roadmap.md`'s **normalization row, `tiler::rms-norm-f32@1`**. The extent is precise and **does not move the rung**: both realization stages are now spelled by scheduled regions that verify, bind their subject, are selected into a complete plan, and agree with `tiler-reference` bit for bit — but no *program* compiles, because the kernel program has no account for the consuming stage. `docs/**` is outside this ticket's scopes; the row edit is routed to the coordinator, and the ledger update belongs beside [`correct-the-one-region-per-occurrence-claim-in-the-records`](correct-the-one-region-per-occurrence-claim-in-the-records.md).

### Checks

`cargo fmt --all --check`; `cargo clippy --all-targets -D warnings` for `tiler-ir` and `tiler-compiler`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` for both; `cargo nextest run --workspace` **2878 passed, 7 skipped**; `cargo test --workspace --doc` green including the ADR 0051 compile-fail evidence; `tkt lint`; `git diff --check`; `tkt guard` against the true base.
