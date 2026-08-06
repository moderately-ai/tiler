---
id: admit-the-registered-elementary-families-as-recognizable-program-stages
title: Admit the registered elementary families as recognizable program stages
status: todo
priority: p1
dependencies: [widen-the-staged-realization-law-to-the-registered-elementary-families, resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage, implement-stage-level-cover-atoms-for-multi-region-occurrences, resolve-which-authority-mints-a-multi-stage-region-candidate, enumerate-region-candidates-over-realization-stages]
related: [accept-the-governed-reciprocal-square-root-scalar-key]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-recognizer
lease_expires_at: 1786041779
---
## User-visible outcome

A program whose middle stage is a registered elementary family — a softmax, an RMS normalization, or any family the registry carries with per-family facts — compiles through the ordinary path when that stage reads a materialized intermediate and feeds a later stage, exactly as elementwise epilogues and strict folds already do. The capability is the general one: any registered family with a realization law becomes a recognizable stage; no family-specific recognizer arm and no chain-shaped special case.

## Why this exists, and the worked instance

**Fact.** The multi-region realization law (accepted 2026-08-06) provides the staged template vocabulary, and its non-goals record that "registering the normalization's or the softmax's own law belongs to the family tickets once this vocabulary exists" — and no such family tickets were ever filed. The elementwise-epilogue, fold-write, and subset-read landings admitted every other stage shape; the registered elementary families are the remaining unrecognizable middles.

**The worked instance is the attention chain** (contraction → softmax → contraction): its three-member region already derives fusion legality, but the softmax cannot be a program stage because it has no registered index-realization law and no `NormalizedOutput` classification. The same wall holds rms-norm-after-anything and every future example program with an elementary middle — which is why this is one general capability, not an attention feature. Per the worked-examples discipline recorded in AGENTS.md: the example exercises the machinery; the capability lands general.

## Sequencing and the boundary inside it

- **The scalar admissions gate the laws, and they are semantic surfaces.** The softmax's law needs an `exp` scalar operation and rms-norm's needs `rsqrt`; `crates/tiler-ir/src/index/scalar.rs` registers ten governed keys and neither. **Corrected 2026-08-06 by [`re-read-the-bf16-and-elementary-support-rows-against-source`](re-read-the-bf16-and-elementary-support-rows-against-source.md), and only half of that sentence survives.** `exp_f32_scalar_op` is `crates/tiler-ir/src/index/scalar.rs:65` — the activation's landing put it there beside `divide_f32_scalar_op` at `:52` — so the softmax's *exponential* needs no new key. The ten keys are `constant`, `multiply`, `add`, `divide`, `exp`, and `canonicalize-nan` at `f32`, the strict-affine U4 dequantize, and the three `bf16` rows. What is genuinely absent is `rsqrt` for the normalization and a **maximum** for the softmax's shifting fold — a second missing key the sentence did not name, and one the softmax's registered definition pins to the NaN-propagating family with `-0.0 < +0.0` rather than leaving open. So the two families need different keys rather than sharing one gap, and the sequencing below should be read against that. A new scalar operation key is a public semantic surface — implemented as a labelled draft, acceptance node parked for Tom, per the standing convention.
- The law registrations then use the accepted staged template (or a single-region law where the family is one region); recognition follows as a `NormalizedOutput` classification derived from the registered law rather than a per-family arm.
- The two-region occurrence tests in `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs` are the harness precedent; the wall test asserting `MissingRealizationLaw` for the normalization flips with the registration.

## Closes when

At least one registered elementary family compiles as a middle stage through the ordinary path with reference bit-agreement, the recognition is law-derived rather than family-cased, the scalar-admission surfaces are parked for Tom, and the attention chain's remaining refusal (if any) names a wall outside this ticket with an owner.

## Progress 2026-08-06 — two of this ticket's premises are refuted by source, and the stop is converted to structure

Both of the stop conditions the dispatch named fired, and each is a fact about the tree rather than an estimate. What landed is the one prerequisite that is fully determined by already-accepted facts; the rest is filed with its derivation.

### Landed

- **`tiler.scalar::rsqrt-f32@1`, as a labelled draft** (`crates/tiler-ir/src/index/scalar.rs`, re-exported from `index/mod.rs`). Unary, homogeneous `f32`, no attributes, sharing the elementary fact record with `exp-f32` through the generalized `elementary_f32_scalar_facts`. Two tests, each watched failing under a deliberate perturbation: `the_reciprocal_square_root_shares_the_elementary_fact_record` (perturbed by registering `arithmetic_f32_scalar_facts` instead) and `the_reciprocal_square_root_refuses_a_foreign_operand_and_a_second_one` (perturbed by declaring `ScalarArity::exact(2)`).
- **Exactly one pinned identity moved**, whole and in the same commit: `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately`, `8966151e455093ea` → `ce6f9106c1c5933b`, with its ledger paragraph. No domain version stepped: `tiler.scalar-registry-snapshot.v1`'s framing and field order are untouched and a definition is self-delimiting, so the row addition is appends-only. No law-sidecar row and no semantic-snapshot row moved with it, and no reached-only projection moved at all — `cargo nextest run --workspace` is 2836 passed with that single pin edited.
- Acceptance parked at [`accept-the-governed-reciprocal-square-root-scalar-key`](accept-the-governed-reciprocal-square-root-scalar-key.md).

### Not landed, and why — the staged template expresses *neither* registered elementary family

**Fact.** The bullet above claiming "the law registrations then use the accepted staged template" is false as written. `IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32` (`crates/tiler-ir/src/index/law.rs:106-111`, realized at `:953-1017`) folds operand zero with **no prologue** and applies exactly **one binary** scalar to operand one and the fold. The normalization needs a fold over `x_i * x_i`, an epilogue chain on the intermediate (`/ N`, `+ eps`, `Rsqrt`), and a *ternary* pointwise stage — and would silently drop its `eps` attribute if the template were registered for it, because `reduction_axes` reads by field ID and tolerates extra fields (`law.rs:1396-1402`). The softmax needs two distinct folds, the first with no registered scalar combiner at all.

So the dispatch's fallback — "land the normalization half if it stands alone" — does not apply: the normalization half does not stand alone in the accepted vocabulary either. Filed with the full derivation as [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md), and the softmax's missing scalar as [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md) — split out rather than guessed because its `SCALAR_FACT_NAN_RESULT_RULE` needs a third value in a published vocabulary, which is a decision rather than a registration.

### Not landed, and why — law-derived recognition is an architecture fork, not a widening

**Fact.** Region attribution is keyed on the exact set of semantic occurrences a region covers, and nothing finer exists: `NormalizedOutput::owns_region_members` (`crates/tiler-compiler/src/request.rs:1660-1680`) distinguishes an output's parts by *disjoint member sets*, `physical::spell_output` (`crates/tiler-compiler/src/physical.rs:442-518`) resolves a placed region by `members ==` and answers with the first matching arm, and `cover::derive_duplication` (`crates/tiler-compiler/src/cover.rs:1999-2018`) reads a repeated member as duplication of that occurrence rather than as a split of it.

**Inference.** One elementary occurrence realizing as two regions gives both regions the same member set, so the second is unreachable. The existing epilogue chain works only because its parts come from different occurrences — `sum(x * x) * scale` is three occurrences the walk splits, `rms_norm(x, w)` is one. `RegionWrite` does not disambiguate them: `spell_output` matches members first, and the publishing-copy second dispatch copies a computed value rather than computing a second stage. Two coherent resolutions exist with different priorities; per AGENTS.md both are drafted and parked at [`resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`](resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md).

**Discharged 2026-08-06.** Tom chose Option A at the live session and [`implement-stage-level-cover-atoms-for-multi-region-occurrences`](implement-stage-level-cover-atoms-for-multi-region-occurrences.md) landed it: the attribution atom is now `SemanticStage`, a *(member, stage)* pair, at every site this section cites — `owns_region_members`, `spell_output`, `derive_duplication`, the region graph, and program assembly's stage coverage. A repeated member at distinct stages is a split rather than a duplication, and a split reduction's combine names the reduction occurrence's second stage instead of claiming nothing. **This wall is no longer what blocks law-derived recognition; what remains for this ticket is the recognizer half** — a `NormalizedOutput` classification derived from a registered multi-region law, which needs the law the other dependency supplies. One obligation was deliberately left behind rather than guessed: region content, region occurrence, and request-subject identity still encode occurrences alone, which is complete while every candidate is single-stage and is refused by name (`unencoded-member-stage`) once one is not. The ticket that mints the first multi-stage candidate must land the encoding whole — [`fold-the-attribution-stage-into-region-and-request-subject-identity`](fold-the-attribution-stage-into-region-and-request-subject-identity.md).

### The attention chain's remaining refusal, and its owner

`softmax_recognizer_boundary.rs` still refuses under every contract, and the refusal is now attributed rather than described: the softmax has no registered realization law (owned by the law-widening ticket, itself waiting on the maximum key) and no representation as a multi-region stage (owned by the attribution fork). Neither wall is inside this ticket's scopes-as-dispatched, and neither is a property of the attention chain — `rms-norm`-after-anything hits both identically, which is what keeps this the general capability the outcome states rather than an attention feature.

### What remains here

This ticket's stated outcome is unsupported until the law widening and the attribution fork both land; those are its dependencies. The four checks it named are otherwise unchanged.

## Progress 2026-08-06 (second round) — the family's lowering landed; the recognizer half is blocked on a fork the graph does not carry

### Landed

**`GovernedRootMeanSquareScaleF32`** (`crates/tiler-compiler/src/governed.rs`), the governed profile's tenth index-access capability and the first whose realization is a region *sequence*. It emits the chain `IndexRealizationLaw::StagedRootMeanSquareScaleF32` pins — a squaring fold, the `/N`, `+eps`, `Rsqrt` epilogue inside the producing stage, then the ternary weighted pass — and `refine_index_region` accepts it, which is a byte-for-byte canonical region-sequence identity match against the law rather than a structural resemblance.

Supporting generalizations, each because the law generalized the same way and the two emissions must agree axis for axis: `SumPlan` gained `for_boundaries` (a fold whose published shape is a parameter, since a staged fold's result is nobody's occurrence result), `squaring_contributors`, a `square` step applied at both contributor reads, and a `fold` method that is now the single statement of the fold — `GovernedStrictSerialSumF32::lower` calls it instead of restating the empty/singleton/tail cases. `reduction_axes` takes its field identifier as a parameter, because attribute identifiers are record-local and both families number an axes field one.

**Evidence.** `governed::tests::the_governed_normalization_lowering_refines_its_two_stage_occurrence`, which asserts the chain rather than success: two stages, one rank-zero-per-row intermediate at `[2]` from a `[2, 4]` occurrence reduced on axis one, the two stages' *different* reached scalar authorities (the fold reaches add, constant, divide, multiply, rsqrt; the pass reaches multiply alone), and operand bindings `[(0, 0), (0, 1), (1, 1)]` — the value operand read by both stages, the weight by the pass alone. Watched failing under a deliberate perturbation: dropping `.squaring_contributors(multiply_f32_scalar_op())` so the fold folds `x` rather than `x²`, which fails with `IrVerifier(SemanticRealizationSequenceMismatch { .. })`.

**One pinned identity moved, whole and in the same commit:** `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately`, `b88654bff9b673c1` → `6f153efeb2da5bb1`, recomputed on this tree from the observed failing value. The request subject binds the canonical lowering-registry identity, which gained this capability's row. No encoding version stepped and no other pin moved: `cargo nextest run --workspace` is 2848 passed, 7 skipped with that single value edited.

**In-crate records rewritten in place to current truth**, because both stated the superseded limit as a live wall: `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs`'s header (its normalization half is no longer a ceiling, and its `the_normalization_resolves_its_law_and_is_held_by_what_a_provider_emits` is now `a_substituted_provider_for_a_shipped_staged_family_reports_its_own_refusal` — the same assertions, reframed as the substitution affordance they actually exercise) and `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`'s header. Four records *outside* these scopes still state it and are filed as [`correct-the-one-region-per-occurrence-claim-in-the-records`](correct-the-one-region-per-occurrence-claim-in-the-records.md).

### Not landed, and why — the recognizer half needs a decision nobody has made

**Fact.** A recognized part is compared to a cover region's atoms by exact equality (`NormalizedOutput::owns_region_members`, `physical::spell_output`), so a partition naming a non-first stage is unusable unless a region *candidate* carries that atom. **Fact.** `region::assemble` is the sole constructor of a `RegionCandidate` and mints only first stages, and it states why: it observes the semantic DAG, where an occurrence is one operation. **Fact.** Splitting one occurrence into two candidates needs the region graph to carry a value the program does not have — the published `r` is inside the realization, not in the program's value list — which propagates to `RetainedOutput`, `MaterializationEdge::value`, program assembly's internal values, and the ABI. **Fact.** `verify_cover` would refuse such a cover anyway, counting per operation and reading two as `IllegalDuplication`; `cover::member_index`'s own doc records this and names the obligation.

So the question "which authority mints the multi-stage claim" is open, two coherent answers exist, and they disagree about whether an identity domain moves at all — which is why the encoding half below reports *not fired* rather than fired. Drafted with the derivation and parked at [`resolve-which-authority-mints-a-multi-stage-region-candidate`](resolve-which-authority-mints-a-multi-stage-region-candidate.md), which is now this ticket's dependency.

### What remains here

This ticket's stated outcome — the normalization compiling as a middle stage with reference bit-agreement — is unsupported until that fork is decided and implemented. What moved is the wall the tree's own harness named: `tiler::rms-norm-f32@1` had a registered law and no provider, and now has both. The four checks this ticket named are otherwise unchanged.

## Re-evaluation — 2026-08-06, after the A′ enumeration landed

[`enumerate-region-candidates-over-realization-stages`](enumerate-region-candidates-over-realization-stages.md) is `done`: staged candidates enumerate, the cover search covers every stage exactly once, and the identity encoding landed whole with zero moved pins. What remains for this ticket's outcome is exactly two walls, both fail-closed today: the **recognizer half** — `select_supported_strategy` still refuses a staged family under `operation-set` before formation runs, so no whole program reaches the staged machinery through `compile()`; and the **physical half** — a selected split-stage cover's handed value has no materialization through program assembly and the ABI. The first is this ticket's own remaining work; the second becomes concrete only once the first admits a program whose selected cover splits.
