---
id: repair-the-research-records-the-key-replacement-and-splits-falsified
title: Repair the research records the key replacement and splits falsified
status: todo
priority: p2
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified, repair-the-navigation-and-contract-docs-the-audit-falsified, repair-the-ticket-population-facts-the-splits-and-retirements-falsified]
scopes: [research/scheduling, research/reference, research/indexing, research/shapes, research/program-planning, research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, audit, research]
---
## User-visible outcome

No live research record states, as a **Fact**, a constant that was deleted, a vocabulary size that has grown, a storage capability that now exists, or a module path that no longer resolves. Adopted and complete records in particular stop presenting retired code as current.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and re-verified site by site by the coordinator at `de18ebdb`.

**Fact — an adopted, complete research record states a Fact naming two constants that no longer exist.** `docs/research/scheduling/cpu-vector-lane-tier.md` (frontmatter `research_status: "complete"`, `disposition: "adopted"`, `adopted_by: ["ADR-0093"]`), anchor `` both `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` ``. The sentence is labelled **Fact**, cited to `contraction_f32_facts` in `crates/tiler-ir/src/semantic/contraction.rs`, and asserts the family declares both constants as `false`. The cited function still exists; the constants do not — `grep -rn "CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED\|CONTRACTION_F32_FACT_PERMUTATION_PERMITTED" crates/` returns nothing at this base. `reduction_descriptor_record` in that same file now declares the row `"permission-gated"`. **The record already carries an inline ADR-0112 parenthetical that repaired only the key spelling and left the two dead constants standing beside it** — a correction that made the record look swept. Repairing the remaining half is the point of this entry, and the same shape should be looked for wherever an ADR-0112 note already exists.

**Fact — `docs/research/reference/plan-freedom-sites.md` states two stale counts and pins them by line number.** Anchor `has eight variants and is deliberately`. `ScalarProgram` (`crates/tiler-ir/src/schedule/model.rs`, anchor `pub enum ScalarProgram`) has **nine** variants, not eight; the same sentence's `ReductionTopology` "five variants" is **seven** (`None`, `Serial`, `MultiPass`, `Contraction`, `LiveContraction`, `CooperativeWorkgroup`, `CooperativeContraction`). The paragraph also cites `:459`, `:715`, `:987`, `:628`, and `:1006` in a file whose real definitions are elsewhere — replace the line pins with searchable anchors while repairing the counts, per the AGENTS.md citation rule. Its claim that both halves are exhaustive enums not marked `#[non_exhaustive]` is separate; verify it rather than carrying it.

**Fact — `StorageScalar` gained an integer carrier, and four live records still deny it.** `StorageScalar` (`crates/tiler-ir/src/program/model.rs`, anchor `pub enum StorageScalar`) has **four** variants — `U8`, `F32`, `Bf16`, and `U32`, the last documented "An unsigned 32-bit integer carrier". In these scopes:

- `docs/research/program-planning/complete-model-ingestion-and-execution.md` (`pending`), anchor `none of the three is an integer carrier of eighteen bits`.
- `docs/research/program-planning/model-level-qualification.md` (`pending`), anchor `carries no integer carrier wider than a byte` — **this row's "No" verdict rests on the retired fact**, so repairing the sentence changes what the row concludes; say so explicitly rather than editing the prose and leaving the verdict. Its own 2026-08-07 inline correction is itself one widening behind, which is the same made-to-look-swept shape as the entry above.

**Fact — research records still name the retired contraction key as current.** ADR 0112 replaced `tiler::strict-tensor-contraction-f32@1` with a permission-indexed successor, and `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs` pins that the old key never compiles. Still present tense in these scopes:

- `docs/research/shapes/transformer-operation-and-shape-surface.md`, anchor `under the single key`. Another line of the same file did receive an ADR-0112 note; this one did not.
- `docs/research/program-planning/flash-class-capability-set.md`, in its worked program; no ADR-0112 note anywhere in the file.
- `docs/research/region-search/rewrite-search-formalism.md` (`adopted`), same shape, in its worked program.

Six further research records carry **correct** forward-references and state the retired key as history — `bf16-computation-accumulator-and-conversion`, `first-attention-program-vertical`, `first-metal-lm-workload`, `general-compilation-boundary`, `first-metal-contraction-realizations`, and the compile-profile authority ledger. Do not "repair" those; they are the model to match.

**Fact — research records cite module paths deleted by the splits.** `crates/tiler-ir/src/schedule/builder.rs` does not exist (now `schedule/builder/`, with `contraction.rs`, `copy.rs`, `coverage.rs`, `diagnostics.rs`, `elementwise.rs`, `family.rs`, `intrinsic.rs`, `mod.rs`, `proof.rs`, `reduction.rs`, `structural_relation_tests.rs`, `tests.rs`). Cited in `docs/research/scheduling/cpu-vector-lane-tier.md`, `multi-round-two-level-reduction-composition.md`, `two-level-subgroup-workgroup-reduction.md`, `two-dimensional-cooperative-staging-relation.md`, `subgroup-execution-tier.md`, `docs/research/reference/plan-freedom-sites.md`, and `docs/research/reference/permitted-divergence-oracle.md`. `crates/tiler-ir/src/index/refinement.rs` does not exist either (now `index/refinement/`); cited in `docs/research/indexing/index-access-model.md`, anchor `and \`index/refinement.rs\` carries`, inside a paragraph headed **Implemented support with a tested guarantee**. `crates/tiler-compiler/src/request.rs` **still exists** beside its new `request/` submodules, so citations to that path are not automatically stale — but named symbols may have moved. Treat path and symbol separately.

Known symbol relocations verified at this base, offered so the worker does not re-derive them: `reads_bind_boundary_tensors_in_order` → `schedule/builder/elementwise.rs`; `split_family` → `schedule/builder/family.rs`.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict before editing. Re-derive counts by reading each enum body.
- Repair each site with a dated correction in the file's own convention. Where a repair changes what a record **concludes** (the `model-level-qualification` row above), state the changed conclusion explicitly; do not leave a repaired premise under an unrepaired verdict.
- Wherever an ADR-0112 or similar inline note already exists, read the whole surrounding claim — this ticket exists partly because two such notes repaired only the cheapest half of what they touched.
- Census the deleted-path citations across these scopes with `grep -rlF` and quote the counts, excluding `docs/research/documentation/ticket-audit-2026-08-10/**`, which is dated history and is not repaired.
- Replace line-number pins with searchable anchors at every site you touch, and run each anchor's grep against the file its citation names before writing it.

## Non-goals

`docs/decisions/**`, navigation and contract documents, ticket bodies, `docs/research/target-profiles/**`, and any source change — each is another ticket's scope. Do not re-litigate ADR 0112 or the `U32` carrier's admission; this ticket records their consequences.

## Closes when

Every site above is repaired or verified already-correct with evidence, the path censuses are quoted with counts, any changed conclusion is stated as such, `make citations` is green, and no live record in these scopes states a deleted constant, a stale vocabulary size, or the retired contraction key in the present tense.
