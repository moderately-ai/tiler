---
id: close-the-gather-review-findings-on-the-index-layer
title: Close the gather review findings on the index layer
status: done
priority: p1
dependencies: []
related: [admit-the-selected-data-dependent-index-representation]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, verification, test-coverage]
---
## User-visible outcome

The independent oracle's gather evaluation is executed by tests rather than trusted by inspection; the whole-region revalidator checks the rule set its own documentation claims; and the three "sourced" negative controls can actually discriminate the failure they are cited for.

## Why this exists

Filed 2026-08-22 from the independent review of `admit-the-selected-data-dependent-index-representation` at `3e04a21c`. The review found **no identity defect** — it independently re-derived that `tiler.index-region.v11` correctly does not step and that the gather encoding is injective, by reading the encoder rather than re-running the author's probe. These are the findings that survived. **Every Fact below was re-verified by the coordinator at `fe3ea025` with the command shown.**

**F1 — HIGH. The reference oracle's gather evaluation has zero test coverage.** `crates/tiler-reference/src/oracle.rs` gained ~197 net lines — `RegionEvaluation::gather`, `dense_input`, the reworked `access_offset`, `IndexRegionEvaluationError::GatherIndexOutOfBounds` — and nothing executes the gather path.

> **Imprecision, 2026-08-22, worker-gatherfix.** "nothing executes any of it" overreaches on two of the four names. `access_offset` and `dense_input` are both reached by the pre-existing direct-read cases in `crates/tiler-reference/tests/index_region_oracle.rs`; it is the **gather** entry and its own error that nothing executed. Two nearby traps for a re-auditor: `grep GatherIndexOutOfBounds crates/` returns nine sites, but the eight outside `oracle.rs` belong to `ReferenceOperationError`, a different enum on the operation-level path in `structural.rs` that `tests/gather_conformance.rs` already covers. And `index_region_oracle.rs` already contained helpers named `gather_region` and `gather` before this work — they author **affine direct reads**, not `gather_read`, so their presence is not coverage of anything here.

- `grep -rno "IndexRegionEvaluationError::GatherIndexOutOfBounds" crates/ | wc -l` → **1**, the construction site itself.
- `grep -rn "gather_read" crates/tiler-reference/ | wc -l` → **0**.
- `git diff --name-only 3cca5438..3e04a21c -- crates/tiler-reference/` → **`src/oracle.rs` only**; no test file.

This matters more than a coverage number: **the oracle is the independent check on what a gather means.** Four semantics can be wrong and return a *wrong element* rather than an error — the big-endian `u32::from_be_bytes` decode; `coordinates.insert(axis, selected)` splicing the loaded address into the source-coordinate run; the row-major linearization; and the ordering that puts the bounds decision before the source read. A wrong `insert` position is silent. The review reasoned the semantics are correct by inspection and cross-checked them against `structural.rs::u32_elements` and `gather_result_shape` — that is an argument, not evidence.

**F2 — MEDIUM. The whole-region revalidator's doc claims a rule-for-rule mirror it does not provide.** `verify_gather_access`'s doc reads "Revalidates one gather access against **every** rule `gather_read` enforces." It does not revalidate distinctness under its own name: `gather_read` raises `IndexBuildError::GatherAliasedTensors` when source and index name one tensor, but `grep -c GatherAliasedTensors crates/tiler-ir/src/index/builder/proof.rs` → **0**, and `GatherAccessRule` has no alias member.

> **Correction, 2026-08-22, worker-gatherfix.** The clause that followed here read "So a corrupted `AccessData` naming one tensor in both roles passes whole-region verification." **That is false, and the worker must not restore it.** An aliased access names one tensor, which carries one value type, so the two type rules cover every aliasing between them: aliasing onto the f32 source leaves the index role holding f32 and `GatherAccessRule::IndexType` fires; aliasing onto the u32 index leaves the source role holding u32 and `GatherAccessRule::SourceType` fires. There is no third case, and nothing corrupt passes. Demonstrated in `crates/tiler-ir/src/index/builder/proof.rs` by `an_alias_onto_either_operand_is_refused_by_a_type_rule`, which drives both aliasings; deleting either type arm reddens it. The finding that survives is the **doc**, not a correctness gap.

Separately, none of the `refuse` arms has ever run — the arms number **15 call sites over 14 distinct rules**, not 14 arms, because `DomainShape` is raised from two sites — — `IndexRegionDiagnostic::GatherAccess` appears twice, both production sites — because every gather reaching `verify` came through `prepare_gather_access`, which already enforced the same rules. That unreachability is by design, but AGENTS.md requires stating what it would take to say *no* and confirming that case is reachable; today nothing demonstrates any rule fires.

**F3 — MEDIUM. The three "sourced" negative controls cannot discriminate.** `crates/tiler-ir/tests/index_gather.rs`'s `refusal()` doc claims "The environment here `determines` every symbol", and the test asserts a rule consulting it "would derive a concrete shape and report a shape disagreement — or worse, admit the region". Verified false: `scale_environment` calls `declare` and `bind` and **never** `require` — `grep -c "\.require(" crates/tiler-ir/tests/index_gather.rs` → **0**. An unconstrained symbol carries the default interval and `determined_extent` returns `None`, which `crates/tiler-ir/src/shape/env.rs`'s own test pins with the message `an unconstrained symbol is not a convenient value`. So an implementation that wrongly consulted `determined()` instead of `as_static()` would refuse anyway and all three controls would stay green. **The behaviour is correct — the review verified the refusal order by reading `prepare_gather_access` in full — but the cited evidence does not bear the weight.**

**F4 — LOW-MEDIUM. A published rule nothing can raise, with a hand-sized census.** No production site constructs `GatherAccessRule::BoundsResolution`. `the_gather_rule_vocabulary_is_publicly_inspectable` lists 15 variants **by hand** and asserts pairwise inequality of fieldless variants — a tautology under derived `PartialEq` that passes unchanged if the vocabulary grows or shrinks. AGENTS.md: size enumerations from the type. `core::mem::variant_count` is already used in this crate's `builder/tests.rs`.

**F5 — LOW. Five of thirteen new refusals have no control:** `GatherSourceNotInput`, `GatherIndexNotInput`, `GatherSourceRankZero`, `GatherSourceCoordinateRank`, `GatherIndexCoordinateRank`. The packet fixes an exact precedence; five positions in it are unobserved.

**F6 — LOW. An accepted-packet clause was dropped without record.** The packet requires that in the oracle "a static resolution independently checks its proof identity and still bounds-checks defensively". `bounds_resolution`, `statically_proved`, and `invocation_validation_required` appear nowhere in `crates/tiler-reference/`. The defensive bounds check *is* present in both cases, so a wrong proof surfaces loudly rather than silently — but the clause is neither implemented nor listed in the carrier ticket's remainder.

**F7 — COSMETIC. Mangled failure text.** `the_gather_access_frame_pins_its_exact_field_order`'s closing assert message carries two runs of ten literal spaces from a dropped `\` continuation, confirmed with `od -c`. It is the failure text of a check.

**F8 — NIT.** `AccessData::coordinate_ordinals()` returns `Vec<u32>` (a clone for the direct arm) where the previous code borrowed a slice, and is called per access inside `compact`'s reachability loop and `visit_access_dimensions`.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict; re-run each command rather than trusting the output pasted here.
- **F1 is the priority.** Add evaluated `gather_read` coverage: a rank-2 source at axis 0 **and** axis 1 with hand-computed expected output, and an out-of-range index yielding `GatherIndexOutOfBounds` with its exact `index_offset`. Then **perturb the subject** — change `insert(axis, …)` to `push` — and quote the failure. A test that cannot catch the silent-wrong-element case has not closed F1.
- **F2**: decide whether the alias rule joins `GatherAccessRule` or the doc is narrowed to what the function actually checks. Either is defensible; **state which and why**, and do not leave a doc claiming a check it does not perform. Then make at least one `refuse` arm demonstrably fire from a corrupted `AccessData` in a `#[cfg(test)]` unit.
- **F3**: add the missing `require` so the environment genuinely determines its symbol, then confirm the three controls still pass **and** that they now discriminate — the point is that a wrongly-`determined()`-consulting implementation must fail them. If adding the constraint changes what a control proves, say so.
- **F4**: size the census from `variant_count`. **F5**: add the five missing controls, or record which are genuinely unreachable and why. **F7**: repair the failure text. **F8**: restore borrowing if it is free to do so; if not, record the measurement and leave it.
- **F6**: implement the clause or record it in the carrier ticket's `## Remaining work`. Do not leave it in neither place.

## Non-goals

Anything past the index layer — the realization law, schedule relation, KIR wall, compiler normalization, and the ADR 0108 amendment belong to `admit-the-selected-data-dependent-index-representation`'s remainder. Do not re-open the accepted option B surface. **No identity may move**: the review confirmed `tiler.index-region.v11` correctly does not step and the encoding is injective — if any pin or golden moves, stop and report.

## Closes when

The oracle's gather path is executed with a hand-computed expectation and a quoted subject perturbation, the revalidator's doc and behaviour agree with at least one arm demonstrably firing, the three controls discriminate, the rule census is type-sized, F5's five refusals are covered or recorded as unreachable, F6 is implemented or recorded, and the touched packages' gates are green with no pin moved.
