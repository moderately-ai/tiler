---
id: repair-the-research-records-the-key-replacement-and-splits-falsified
title: Repair the research records the key replacement and splits falsified
status: in-progress
priority: p2
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified, repair-the-navigation-and-contract-docs-the-audit-falsified, repair-the-ticket-population-facts-the-splits-and-retirements-falsified]
scopes: [research/scheduling, research/reference, research/indexing, research/shapes, research/program-planning, research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, audit, research]
claimed_from: todo
assignee: worker-research
lease_expires_at: 1787162948
---
## User-visible outcome

No live research record states, as a **Fact**, a constant that was deleted, a vocabulary size that has grown, a storage capability that now exists, or a module path that no longer resolves. Adopted and complete records in particular stop presenting retired code as current.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and re-verified site by site by the coordinator at `de18ebdb`.

**Fact — an adopted, complete research record states a Fact naming two constants that no longer exist.** `docs/research/scheduling/cpu-vector-lane-tier.md` (frontmatter `research_status: "complete"`, `disposition: "adopted"`, `adopted_by: ["ADR-0093"]`), anchor `` both `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` ``. The sentence is labelled **Fact**, cited to `contraction_f32_facts` in `crates/tiler-ir/src/semantic/contraction.rs`, and asserts the family declares both constants as `false`. The cited function still exists; the constants do not — `grep -rn "CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED\|CONTRACTION_F32_FACT_PERMUTATION_PERMITTED" crates/` returns nothing at this base. `reduction_descriptor_record` in that same file now declares the row `"permission-gated"`. **The record already carries an inline ADR-0112 parenthetical that repaired only the key spelling and left the two dead constants standing beside it** — a correction that made the record look swept. Repairing the remaining half is the point of this entry, and the same shape should be looked for wherever an ADR-0112 note already exists.

**Fact — `docs/research/reference/plan-freedom-sites.md` states two stale counts and pins them by line number.** Anchor `has eight variants and is deliberately`. `ScalarProgram` (`crates/tiler-ir/src/schedule/model.rs`, anchor `pub enum ScalarProgram`) has **nine** variants, not eight; the same sentence's `ReductionTopology` "five variants" is **seven** (`None`, `Serial`, `MultiPass`, `Contraction`, `LiveContraction`, `CooperativeWorkgroup`, `CooperativeContraction`). The paragraph also cites `:459`, `:715`, `:987`, `:628`, and `:1006` in a file whose real definitions are elsewhere — replace the line pins with searchable anchors while repairing the counts, per the AGENTS.md citation rule. Its claim that both halves are exhaustive enums not marked `#[non_exhaustive]` is separate; verify it rather than carrying it. *Imprecise, corrected 2026-08-19 by the worker at `f08281a1`: the record does not make that claim. It says the two are "exhaustive enums **inside their defining crate**" and marks only `ScalarProgram` as deliberately not `#[non_exhaustive]`. Read at this base, `ScalarProgram` is indeed unmarked and `ReductionTopology` **is** `#[non_exhaustive]` under ADR 0074 convention 5a — which leaves the record's sentence true as written, because the attribute has no effect within `tiler-ir`. The repair therefore states the asymmetry so a reader cannot carry the unmarked half across, rather than retiring a claim that was never made.*

**Fact — `StorageScalar` gained an integer carrier, and four live records still deny it.** `StorageScalar` (`crates/tiler-ir/src/program/model.rs`, anchor `pub enum StorageScalar`) has **four** variants — `U8`, `F32`, `Bf16`, and `U32`, the last documented "An unsigned 32-bit integer carrier". In these scopes:

- `docs/research/program-planning/complete-model-ingestion-and-execution.md` (`pending`), anchor `none of the three is an integer carrier of eighteen bits`.
- `docs/research/program-planning/model-level-qualification.md` (`pending`), anchor `carries no integer carrier wider than a byte` — **this row's "No" verdict rests on the retired fact**, so repairing the sentence changes what the row concludes; say so explicitly rather than editing the prose and leaving the verdict. Its own 2026-08-07 inline correction is itself one widening behind, which is the same made-to-look-swept shape as the entry above.

**Fact — research records still name the retired contraction key as current.** ADR 0112 replaced `tiler::strict-tensor-contraction-f32@1` with a permission-indexed successor, and `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs` pins that the old key never compiles. Still present tense in these scopes:

- `docs/research/shapes/transformer-operation-and-shape-surface.md`, anchor `under the single key`. Another line of the same file did receive an ADR-0112 note; this one did not.
- `docs/research/program-planning/flash-class-capability-set.md`, in its worked program; no ADR-0112 note anywhere in the file. **False, corrected 2026-08-19 by the worker at `f08281a1`.** The file *does* carry a note — `grep -nF 0112 docs/research/program-planning/flash-class-capability-set.md` returns one line, the comment `(key spelling at this record's date; ADR 0112 renamed it tiler::tensor-contraction-f32@1)` inside the worked program's own fenced block. The real defect was narrower: the block spells the key on two lines and the note sat against only the first, so the note's *reach* was widened rather than a missing note added.
- `docs/research/region-search/rewrite-search-formalism.md` (`adopted`), same shape, in its worked program. **False, corrected 2026-08-19 by the worker at `f08281a1`: this record is already correct and needed no repair.** `grep -nF 0112 docs/research/region-search/rewrite-search-formalism.md` returns the same note form on the line directly below the record's single retired-key occurrence, which that occurrence is the whole population of. It belongs with the six the ticket lists as the model to match, not with the sites needing repair.

Six further research records carry **correct** forward-references and state the retired key as history — `bf16-computation-accumulator-and-conversion`, `first-attention-program-vertical`, `first-metal-lm-workload`, `general-compilation-boundary`, `first-metal-contraction-realizations`, and the compile-profile authority ledger. Do not "repair" those; they are the model to match.

**Fact — research records cite module paths deleted by the splits.** `crates/tiler-ir/src/schedule/builder.rs` does not exist (now `schedule/builder/`, with `contraction.rs`, `copy.rs`, `coverage.rs`, `diagnostics.rs`, `elementwise.rs`, `family.rs`, `intrinsic.rs`, `mod.rs`, `proof.rs`, `reduction.rs`, `structural_relation_tests.rs`, `tests.rs`). *Imprecise, corrected 2026-08-19 by the worker at `f08281a1`: the directory holds **thirteen** files, not the twelve listed — `tile.rs` is omitted above, and it is load-bearing for at least one repair, since `two-level-subgroup-workgroup-reduction.md`'s existing note places the workgroup-width equality in `builder/tile.rs`. Re-derive with `ls crates/tiler-ir/src/schedule/builder/`.* Cited in `docs/research/scheduling/cpu-vector-lane-tier.md`, `multi-round-two-level-reduction-composition.md`, `two-level-subgroup-workgroup-reduction.md`, `two-dimensional-cooperative-staging-relation.md`, `subgroup-execution-tier.md`, `docs/research/reference/plan-freedom-sites.md`, and `docs/research/reference/permitted-divergence-oracle.md`. `crates/tiler-ir/src/index/refinement.rs` does not exist either (now `index/refinement/`); cited in `docs/research/indexing/index-access-model.md`, anchor `and \`index/refinement.rs\` carries`, inside a paragraph headed **Implemented support with a tested guarantee**. `crates/tiler-compiler/src/request.rs` **still exists** beside its new `request/` submodules, so citations to that path are not automatically stale — but named symbols may have moved. Treat path and symbol separately.

Known symbol relocations verified at this base, offered so the worker does not re-derive them: `reads_bind_boundary_tensors_in_order` → `schedule/builder/elementwise.rs`; `split_family` → `schedule/builder/family.rs`.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict before editing. Re-derive counts by reading each enum body.
- Repair each site with a dated correction in the file's own convention. Where a repair changes what a record **concludes** (the `model-level-qualification` row above), state the changed conclusion explicitly; do not leave a repaired premise under an unrepaired verdict.
- Wherever an ADR-0112 or similar inline note already exists, read the whole surrounding claim — this ticket exists partly because two such notes repaired only the cheapest half of what they touched.
- Census the deleted-path citations across these scopes with `grep -rlF` and quote the counts, excluding `docs/research/documentation/ticket-audit-2026-08-10/**`, which is dated history and is not repaired.
- Replace line-number pins with searchable anchors at every site you touch, and run each anchor's grep against the file its citation names before writing it.

## Non-goals

`docs/decisions/**`, navigation and contract documents, ticket bodies, `docs/research/target-profiles/**`, and any source change — each is another ticket's scope. Do not re-litigate ADR 0112 or the `U32` carrier's admission; this ticket records their consequences.

## Fact audit and path census, 2026-08-19 at `f08281a1`

Every Fact re-read at this base before any edit; the three corrections above are recorded inline where the false or imprecise claim sits.

| Fact | Verdict | Evidence |
| --- | --- | --- |
| Two constants deleted, `cpu-vector-lane-tier.md` states them as **Fact** | **Verified** | `grep -rn "CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED\|CONTRACTION_F32_FACT_PERMUTATION_PERMITTED" crates/` returns nothing; `contraction_f32_facts` still exists and now carries thirteen fields; `"Field IDs 8 and 9 are retired and never reused."` names them |
| `reduction_descriptor_record` declares `"permission-gated"` | **Verified, and sharper than stated** | Row 4 (reassociation maximum) is `"permission-gated"`; rows 5 and 6 (permutation, signed-zero elimination) are `"unsupported"`. `ContractionF32OrderFreedom` has exactly `Unsupported` and `PermissionGated` |
| `ScalarProgram` has nine variants, not eight | **Verified** | Enum body read at `"pub enum ScalarProgram {"`, lines 560–827; nine variants counted |
| `ReductionTopology` has seven, not five, with the seven named | **Verified** | Enum body read at `"pub enum ReductionTopology {"`, lines 1226–1476; the seven names match the ticket exactly |
| `plan-freedom-sites.md` pins `:459`, `:715`, `:987`, `:628`, `:1006` | **Verified** | All five in the paragraph anchored `has eight variants and is deliberately`; all five rotted; replaced with symbol anchors |
| The record's `#[non_exhaustive]` claim | **Imprecise as characterized** | See the inline correction above |
| `StorageScalar` has four variants including `U32` | **Verified** | `"pub enum StorageScalar {"`: `U8`, `F32`, `Bf16`, `U32`, the last `"An unsigned 32-bit integer carrier."` at tag `0x04` |
| `model-level-qualification.md`'s "No" rests on the retired fact | **Verified, and the verdict moved** | Repaired to **carrier yes, Metal emission no**; derivation in the row and in the Inference below it |
| Retired key present tense in `transformer-operation-and-shape-surface.md` | **Verified** | Line anchored `under the single key`; the file's other occurrence already carried a note |
| Retired key in `flash-class-capability-set.md`, no note in file | **False** | See the inline correction above |
| Retired key in `rewrite-search-formalism.md`, same shape | **False — already correct** | See the inline correction above |
| `schedule/builder.rs` and `index/refinement.rs` deleted; `tiler-compiler/src/request.rs` survives | **Verified**; builder file list imprecise | `ls` on each; `request.rs` exists beside `request/`. `index/builder.rs` likewise survives beside `index/builder/`, which is the same shape and is not stale |

**Path census across the six scopes, excluding `docs/research/documentation/ticket-audit-2026-08-10/**` (outside these scopes in any case).** `spikes/{scheduling,reference,indexing,shapes,program-planning,region-search}` all exist and contain **zero** hits for any of the three patterns.

- `schedule/builder.rs` — **7 files, 22 occurrences**: `plan-freedom-sites.md` 7, `two-dimensional-cooperative-staging-relation.md` 6, `multi-round-two-level-reduction-composition.md` 3, `cpu-vector-lane-tier.md` 2, `two-level-subgroup-workgroup-reduction.md` 2, `permitted-divergence-oracle.md` 1, `subgroup-execution-tier.md` 1. **Six of the seven were already repaired** by `point-the-bare-builder-path-mentions-at-the-split-modules` and `re-anchor-the-schedule-builder-line-citations`; `two-dimensional-cooperative-staging-relation.md` carries a deliberate blanket historical note. One gap repaired here: `multi-round-two-level-reduction-composition.md`'s third occurrence sits *below* a note that scopes itself to "both citations of it above".
- `index/refinement.rs` — **1 file, 1 occurrence**: `index-access-model.md`, unrepaired, inside a paragraph headed **Implemented support with a tested guarantee**. Repaired here.
- `strict-tensor-contraction-f32@1` — **7 files, 9 occurrences**: `flash-class-capability-set.md` 2, `transformer-operation-and-shape-surface.md` 2, and 1 each in `first-attention-program-vertical.md`, `first-metal-lm-workload.md`, `first-metal-contraction-realizations.md`, `rewrite-search-formalism.md`, `cpu-vector-lane-tier.md`. **Only one occurrence was present tense with no note**: `transformer-operation-and-shape-surface.md`'s second. `cpu-vector-lane-tier.md`'s occurrence already carried its note and is not a key defect — its defect was the two constants beside it.

**Reproduce the census** with `SCOPES=(docs/research/{scheduling,reference,indexing,shapes,program-planning,region-search}); grep -rcF "<pattern>" "${SCOPES[@]}" | grep -v ":0"`. The array form is load-bearing: in `zsh` an unquoted `$SCOPES` does not word-split, so a bare-variable spelling searches one nonexistent path and reports zero hits for every pattern — a check that cannot reach its subject.

### Remainder mapped, not done

**`plan-freedom-sites.md`'s twenty-five-site headline is a floor, and closing it is enumeration work outside this ticket's fence.** Repairing `ReductionTopology`'s count from five to seven exposed that `LiveContraction` and `CooperativeContraction` each carry their own `permits_reassociation` and `permits_permutation` fields — the same shape that makes sites 4.1 through 4.4 witnesses — so each is a candidate freedom-site row the table does not have, on the precedent of the record's own correction 4 admitting site 4.9 for the ninth `ScalarProgram` variant. Classifying them requires applying the record's Part 1 rule and deriving each spend population, which is a different kind of work from citation repair. The repair states the count as a floor and says the enumeration was not re-run, rather than asserting no bucket moved; **a narrow follow-up ticket should re-run the enumeration over the current vocabulary.** A second, smaller inconsistency was observed and not repaired: the record's closing section still says "The twenty-four sites are exhaustive over the vocabulary", against Part 7.5's twenty-five — a pre-existing mismatch this ticket did not introduce and does not fence.

## Closes when

Every site above is repaired or verified already-correct with evidence, the path censuses are quoted with counts, any changed conclusion is stated as such, `make citations` is green, and no live record in these scopes states a deleted constant, a stale vocabulary size, or the retired contraction key in the present tense.
