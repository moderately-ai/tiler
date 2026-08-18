---
id: carry-the-elementary-numerical-dimensions-in-the-region-realization
title: Carry the elementary numerical dimensions in the region realization
status: in-progress
priority: p2
dependencies: [carry-every-realized-dimension-through-region-feasibility]
related: [admit-the-silu-activation-family, admit-the-rms-normalization-family, admit-the-softmax-family, require-both-elementary-evidence-halves-before-target-admission]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/reference, implementation/metal, contracts/decisions, implementation/build, implementation/conformance, implementation/runtime, implementation/cache, research/target-profiles, contracts/artifacts, implementation/candle, research/reference, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity, feasibility, decision, needs-tom, public-boundary]
claimed_from: todo
assignee: worker-elementary-dimensions
lease_expires_at: 1787069610
---

# Carry the elementary numerical dimensions in the region realization

## User-visible outcome

A caller that states a numerical contract permitting reciprocal replacement or approximate elementary functions gets an answer about a *target*, rather than a build-level refusal or a silently unasked question, for a program containing `tiler::silu-f32@1`.

## Why this is filed rather than done

**Fact — corrected 2026-08-09.** The obligation is no longer activation-only. `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, and `tiler::softmax-f32@1` are all admitted elementary families; each can consume `NumericalDimension::ReciprocalTransform` and `NumericalDimension::ApproximateIntrinsics`. `ELEMENTARY_UNCARRIED_DIMENSIONS` names the common omission explicitly.

**Fact — listing them would refuse a public preset for every program.** `dimension_requirements` filters by `is_consumable`, so a row entry enters the dimension into the requirement set every contract places on every target; and `unrepresentable_dimension` refuses any consumable dimension that `tiler_ir::schedule::NumericalRealization` cannot carry, which neither is. `session::NumericalContract::RELAXED_F32` authorizes both, so it would become unrepresentable — for programs with no elementary family occurrence as well.

**Fact — the omission is a checked claim rather than a gap.** `policy::ELEMENTARY_UNCARRIED_DIMENSIONS` names both, and `the_uncarried_elementary_dimensions_are_outside_the_realization` fails the moment the region realization grows to carry either, which is the condition under which withholding the rows stops being honest.

**Fact — what holds the obligation meanwhile is backend-local.** `crates/tiler-metal/src/emit.rs` writes `precise::exp`, `precise::rsqrt`, and the `/` operator rather than a fast intrinsic or a reciprocal multiply, and records `MetalNumericalRequirement::PreciseFp32Functions`. That is a guarantee over the operations actually emitted; it is not a profile-level assessment, and it speaks for one backend.

## Required delivery

- Widen `tiler_ir::schedule::NumericalRealization` to carry the reciprocal-transform permission and the approximate-intrinsic envelope, each encoded by an exhaustive match over a non-`#[non_exhaustive]` vocabulary, in `schedule::model::push_numerical` and `kernel::model::push_numerical` alike. This is ADR 0076 item 1 and item 6's shape and the two must land together: a widened realization with an incomplete encoder gives two semantically different regions one identity.
- Carry both forward through `ResourceRequirements`, `region_proposal`, and the artifact envelope's `NumericalFacts`.
- Site both dimensions explicitly at the existing `PolicyLocus::Computation` for each consuming composite occurrence. The subordinate division, exponential, or reciprocal square root is arithmetic inside that semantic occurrence; `OperationNumericalCapability::founded_locus` must state that derivation rather than retaining its current `None` or substituting a fallback in the delivered-record producer.
- Add both to `REALIZED_DIMENSIONS` and to all three elementary families' capability rows, and delete `ELEMENTARY_UNCARRIED_DIMENSIONS` and its superseded omission test.
- Declare both on the governed target profile, and decide what the measured Apple row honours for each.
- Extend `tiler-reference`'s exhaustive realization bridge with typed refusals for a reciprocal permission or non-`Forbidden` approximation it cannot evaluate, and extend `tiler-metal`'s realization-to-emission checks so neither new field can compile while being ignored.
- Step whatever identity domains the encoders' record layouts require, and rebaseline every pinned identity on the same tree.

## Non-goals

Widening `MaterializationRounding`, which no admitted operation consumes. Changing the already-live elementary-accuracy assessment path or admitting another semantic family: `request::require_elementary_accuracy` already calls `assess_program_elementary_accuracy`; this ticket carries the generic dimensions rather than creating that assessment.

## Reconsideration trigger

**Fired.** Three elementary families are admitted and the public `RELAXED_F32` preset authorizes both withheld dimensions. This is current implementation work, not a deferred trigger.

## Decision packet — 2026-08-09

The trigger establishes need, but the work changes public `NumericalRealization`, target requirements, artifact facts, and several identity domains. Recommendation: add explicit reciprocal-transform permission and approximate-intrinsic envelope fields, using the already-governed typed behaviours rather than backend booleans, and carry them atomically through schedule, kernel, compiler, and artifact records. Tom must accept that exact cross-layer public shape and its identity step before implementation.

## Accepted decision — 2026-08-11

Tom accepted the revised narrow first pass in the live decision review, relayed first-hand by the coordinator. `NumericalRealization` gains two direct required fields — `reciprocal_transform: NumericalPermission` and `approximate_intrinsics: ApproximationEnvelope` — rather than an optional/defaulted value, inferred profile-key projection, nested elementary bundle, or dynamically typed dimension map. Every constructor must state both. A missing target declaration remains `Unknown` and refuses; reference and backend consumers must either realize the exact typed value or return a typed refusal. There is no compatibility default and no backend retry.

The two obligations use the existing `PolicyLocus::Computation`. Numerical semantics already says the operation's own arithmetic is the locus for dimensions not assigned to input, result, accumulator, component, or materialization, and the subordinate arithmetic remains inside the composite occurrence. This acceptance therefore adds no new locus variant or locus wire tag. It does require repairing `founded_locus`'s current deliberate `None`; the delivered-realization producer must never silently substitute `Computation` for an unfounded row.

The source audit also found a pre-existing prerequisite: `ResourceRequirements` carries eight numerical dimensions while `physical::region_proposal` asks target feasibility about only input/result subnormals, contraction, and reassociation. [`carry-every-realized-dimension-through-region-feasibility`](carry-every-realized-dimension-through-region-feasibility.md) owns restoring the complete existing eight-dimension projection before this ticket grows it to ten. The carrier also explicitly includes the reference and Metal total consumers that the original delivery packet omitted.

At decision base `b48e7719ccd6e9b4919d40bde998beb122bdb9b0`, the expected coordinated layout steps are scheduled region `tiler.schedule.v5` to `v6`, structured kernel `tiler.kernel.v7` to `v8`, artifact program `tiler.artifact-program.v16` to `v17`, and manifest schema `16.0` to `17.0`. The coherent numerical-contract key domains do not step merely for this work because they already encode both dimensions, and the delivered-realization record already carries the complete eleven-dimension vocabulary. Implementation must rederive those expectations and every pin on its own exact base rather than copying these numbers.

**Strongest counterpoint accepted consciously.** The direct fields break every exhaustive constructor and move multiple identity domains. That blast radius is the safety mechanism: old constructors, encoders, reference bridges, and backend projections must stop compiling until they state the new meaning, whereas an optional field or inferred strict value would preserve source compatibility by silently narrowing the caller's contract.

## Source-audit correction — 2026-08-11

The original closure inventory was incomplete in two correctness-bearing directions. `OperationNumericalCapability::founded_locus` explicitly returns `None` for both dimensions, so adding capability rows alone reaches `numerical-realization-locus-unfounded`; and `physical::region_proposal` currently forwards four of the eight fields `ResourceRequirements` already carries. `ReferenceNumericalConformance::from_realization` and Metal's realization checks are additional total consumers that field access could otherwise leave silently incomplete. The accepted packet above supersedes the narrower inventory without changing the ticket's purpose.

## Scope repair — 2026-08-09

`implementation/build` is declared because the required governed Metal profile declaration is owned by `crates/tiler-build`; the prior IR/compiler/artifact scopes could not complete that stated delivery.

## Scope repair — 2026-08-18

Four further scopes are added as scheduling metadata required by the authorized work, none of them a widening of the accepted shape. `implementation/conformance` and `implementation/runtime` are forced by the widened `NumericalRealization`: `crates/tiler-conformance/src/bf16_vertical.rs` (`realization_of`), `crates/tiler-conformance/src/loop_carried.rs` (`declared_realization`), and `crates/tiler-runtime/tests/adapter_route/fixture.rs` construct the record and stop compiling until they state both fields — the acceptance's "every constructor states both" reaches them. `implementation/cache` is forced by the accepted instruction to rebaseline every pinned identity on one tree: the coordinated schedule/kernel/artifact identity steps move cache subjects wherever they are pinned. `research/target-profiles` is forced by the ledger rule in `crates/tiler-build/src/metal_declaration.rs` — "the bound declaration is constructed from exactly those rows and no others" — so declaring the two new measured rows on the Apple profile requires the corresponding rows in `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`; declaring them without ledger rows would break the ledger's own construction constraint. `contracts/artifacts` is forced by the same coherence rule that forces the pin rebaseline: `docs/artifact-abi.md`'s identity ledger states the current schedule, kernel, artifact, and manifest domains by exact version (anchor: `the current identity ledger is source-derived`), and AGENTS.md requires identity-domain changes to stay coherent across owning version, ledgers, and pins. `implementation/candle` is forced by one more constructor: `prototypes/candle-metal-adapter/src/adapter.rs` builds a `ResourceRequirements` fixture and stops compiling until it states both fields; prototypes build under the workspace gate. `implementation/frontend` is forced by the artifact-domain step: `crates/tiler/src/route/tests.rs` deliberately restates `tiler.artifact-program.vN` as `IDENTITY_DOMAIN` (its doc records the restatement as self-detecting), and the `v19` step trips exactly that detection. `research/reference` is forced by the document-alignment obligation: `docs/research/reference/plan-freedom-sites.md` stated in the present tense that the region IR carries eight of the eleven dimensions, which this delivery makes false; it gains a dated correction note rather than a silent edit.

## Fact audit at ec649f4b0da014834a834a23cdee7afd659939bc — 2026-08-18

Per-Fact verdicts, each re-read in full at this base before any edit.

- **"The obligation is no longer activation-only"** — **verified.** `crates/tiler-compiler/src/policy.rs` rows for `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, `tiler::softmax-f32@1`; `ELEMENTARY_UNCARRIED_DIMENSIONS` names both dimensions (anchor: `test-held omission witness until the region realization carries`).
- **"Listing them would refuse a public preset"** — **verified.** `dimension_requirements` filters by `is_consumable` (anchor: `filter(|dimension| is_consumable(*dimension))`); `unrepresentable_dimension` refuses any consumable dimension outside `REALIZED_DIMENSIONS`; `session::NumericalContract::RELAXED_F32` resolves `.reciprocal_transform(NumericalPermission::Permitted)` and `.approximate_intrinsics(ApproximationEnvelope::BackendElementary)`.
- **"The omission is a checked claim"** — **verified.** `the_uncarried_elementary_dimensions_are_outside_the_realization` in `policy.rs` asserts both `!REALIZED_DIMENSIONS.contains` and `!is_consumable`.
- **"What holds the obligation meanwhile is backend-local"** — **verified.** `crates/tiler-metal/src/emit.rs` `emit_unary` writes `precise::exp` / `precise::rsqrt` and inserts `MetalNumericalRequirement::PreciseFp32Functions` and `SafeMathMode`; division emits the `/` operator through the binary path.
- **Accepted decision's expected identity steps ("schedule v5→v6, kernel v7→v8, artifact v16→v17, manifest 16.0→17.0")** — **stale, as the acceptance itself anticipates.** At this base the live domains are `tiler.schedule.v6` (`crates/tiler-ir/src/schedule/model.rs`, `encode_identity`), `tiler.kernel.v8` (`crates/tiler-ir/src/kernel/model.rs`, `KERNEL_DOMAIN`), `tiler.artifact-program.v18` (`crates/tiler-artifact/src/program/model.rs`, `ARTIFACT_DOMAIN`), manifest schema `(18, 0)` (`codec/encode.rs`, `MANIFEST_SCHEMA`), and `tiler.kernel-program.v12`. The rederived steps this delivery takes are recorded below.
- **`founded_locus` returns `None` for both dimensions** — **verified** (anchor: the `founded_locus` arm listing `ReciprocalTransform`, `ApproximateIntrinsics`, and `MaterializationRounding` together over `None` in `policy.rs`), and `crates/tiler-compiler/src/session/realization.rs` refuses an unfounded consumable dimension by `numerical-realization-locus-unfounded` rather than substituting `Computation`.
- **Source-audit correction's "physical::region_proposal currently forwards four of the eight fields"** — **superseded, dependency landed.** `carry-every-realized-dimension-through-region-feasibility` (done) restored the complete eight-dimension projection: `region_numerical_requirements` exhaustively destructures `ResourceRequirements` and projects all eight, with `carried_by_resource_requirements` and a `variant_count`-sized test (`region_proposal_projects_every_realized_resource_requirement`) pinning the population.
- **"`ReferenceNumericalConformance::from_realization` and Metal's realization checks are additional total consumers"** — **verified with a precision.** `from_realization` destructures the realization exhaustively (build error on widening). Metal's `realization_requirements` and `requires_safe_math` read fields by access rather than destructuring, so at this base they would compile while ignoring the new fields — exactly the silent incompleteness the delivery closes by making both exhaustive.
- **Vocabulary existence** — **verified.** `NumericalPermission` (`Forbidden`/`Permitted`) and `ApproximationEnvelope` (`Forbidden`/`BackendElementary`) both exist in `crates/tiler-ir/src/schedule/numerics.rs` under exactly the accepted spellings.
