---
id: derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order
title: Derive the exact evaluator for a multi-round cooperative fold order
status: deferred
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ, accept-adr-0100-multi-round-reduction-composition]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, reductions, conformance]
---
## User-visible outcome

An exact host evaluator for every reduction order a schedule can declare, so that a plan whose topology the reference cannot evaluate is refused by name rather than compared against the nearest order that happens to exist.

## Why this is a deferral rather than work

**Fact — `strict_partitioned_sum` expresses exactly one realization shape.** It folds `partition * chunk + within` (`strict_partial_sums_under` in `crates/tiler-reference/src/evaluate.rs`), i.e. blocked uniform contiguous partitions, serial within a partition, ascending across them, at the element width. `ContributorPartition::covers` admits only an exact product, so a non-uniform split is unrepresentable by construction.

**Fact — shapes outside that oracle still exist in the schedule vocabulary.** `ReductionTopology::CooperativeWorkgroup` (`crates/tiler-ir/src/schedule/model.rs`) documents that on a loop-carried tile "participant `p` of round `r` owns the contiguous range at index `r * partitions + p`" over `partitions * contributors_per_partition * tile.rounds` contributors — a different index map from the flat blocked product above. Multi-round cooperative coverage (rounds > 1) and any future non-uniform split are therefore outside `strict_partitioned_sum`. Both `MultiPass` and `CooperativeWorkgroup` also carry `accumulation: ArithmeticType`, "the width every combining step is performed at", and the declared-order oracle has no such parameter — but that field is not a live unevaluable shape on verified plans (see accumulation correction below).

**Fact — none of the *unevaluable* shapes is reachable from a compiler-constructed plan, and the distinction matters.** Production construction reaches cooperative tiles through `tiler_ir::schedule::workgroup_tree_tile` (`let tile = tiler_ir::schedule::workgroup_tree_tile(participants)` in `crates/tiler-compiler/src/physical.rs`); that constructor hard-codes `rounds: 1` (`fn workgroup_tree_tile` in `crates/tiler-ir/src/schedule/cooperative.rs`, with the "tile carries no anti-dependency" comment). Compiler multi-pass and cooperative construction pin `accumulation: request.numerical_contract().arithmetic` in `physical.rs`. Single-round trees *are* built and compared to `strict_partitioned_sum` (evaluable); multi-round and non-uniform shapes are not.

**Correction — 2026-08-10 (accumulation population).** A verified plan cannot declare accumulation ≠ the region's arithmetic type: `verify_accumulation_width` in `crates/tiler-ir/src/schedule/builder/reduction.rs` refuses `declared != region_arithmetic_type(program)` for both parallel topologies, and `RealizationWitness::accumulation` documents that site 4.8's spend population is empty ("The intrinsic verifier now refuses a declared accumulation that differs from" the region's own arithmetic type). The oracle still lacks a width parameter, but a verified region's declared width *is* its element width. The accumulation trigger limb therefore fires only if the accumulation authority is widened to admit a second width — not merely if the field is written elsewhere or the oracle signature stays width-free.

**Fact — but the schedule vocabulary already admits and verifies a multi-round tile.** `fn multi_round_tile_fixture` in `crates/tiler-ir/src/schedule/builder/tests.rs` constructs a fixture with `rounds: 2` and its tests verify it as a schedule. **Inference — so this deferral is one compiler construction away from firing rather than one ADR away**, which is a materially nearer position than ADR 0100's `implementation_status: not-started` alone suggests, and it is why the trigger check below names the compiler's construction sites rather than searching for a literal.

**Fact — filed `deferred` rather than `todo` because the board must not offer non-work.** There is nothing to evaluate until a topology outside the shape exists on a plan that needs an oracle answer.

**Correction — 2026-08-19 (schedule-builder split; paths repaired, substance re-verified).** The accumulation correction and the multi-round-fixture Fact above both cited `crates/tiler-ir/src/schedule/builder.rs`, which no longer exists — the split replaced it with the `builder/` directory. Both symbols were re-located and both claims re-read and **confirmed unchanged**: `pub(super) fn verify_accumulation_width` is in `crates/tiler-ir/src/schedule/builder/reduction.rs`, and `fn multi_round_tile_fixture` with its `rounds: 2` literal is in `crates/tiler-ir/src/schedule/builder/tests.rs`. The 2026-08-05 entry in the trigger log below also names the old path; that citation is **dated evidence about that base and is retained verbatim** rather than rewritten, since editing it would falsify the record of what that check saw. Its pattern's population has since grown — `grep -rn 'rounds: *[2-9]' crates/ --include='*.rs'` returns **five** hits at this base, not the three recorded there (`schedule/witness/tests.rs` ×2, `schedule/builder/tests.rs`, `pipeline/conformance.rs`, `legality.rs`) — which strengthens rather than weakens that entry's point that an emptiness check over this pattern is the wrong check.

**Trigger recheck — 2026-08-19: not fired.** Production construction still reaches cooperative tiles through the single anchor `let tile = tiler_ir::schedule::workgroup_tree_tile(participants)` in `crates/tiler-compiler/src/physical.rs` (one hit), and `fn workgroup_tree_tile` in `crates/tiler-ir/src/schedule/cooperative.rs` still fixes `rounds: 1`. No compiler path builds a multi-round tile, so no new evaluator is owed. `pub fn strict_partial_sums_under` remains in `crates/tiler-reference/src/evaluate.rs`. Recorded here rather than in the log below because this sweep was a citation repair, not a scheduled trigger evaluation.

**Correction — 2026-08-10 (CooperativeTile census).** An earlier form of this section asserted that `grep -rn 'CooperativeTile' crates/tiler-compiler/src --include='*.rs'` returns exactly three lines all naming `workgroup_tree_tile`. That line-count census is historical (2026-08-05 log); the type name is no longer the production-construction population. Live discipline is the construction anchor above: production path is `workgroup_tree_tile` with `rounds: 1`, not a `CooperativeTile` hit count.

## Trigger

Either: a `ReductionTopology` in `crates/tiler-ir` realizes ADR 0100's multi-round composition with `rounds > 1` reachable from a constructed plan; or the accumulation authority is widened so a verified plan can declare an `accumulation` width other than its element type; or a non-uniform split is admitted.

## What this ticket must produce once fired

- An exact evaluator per newly admitted shape, written as that shape's definition rather than as an approximation of it, with the index map stated and matched against the topology's own documented map.
- A `RealizationNotEvaluable` refusal for a topology no evaluator covers, watched failing.
- A case at which the new shape and the existing blocked shape produce different bits, so the addition is evidence about something.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as refusal class 2's population.

## Trigger check log

- 2026-08-05 — **not fired, and the check names its population rather than relying on an empty result.** `grep -rn 'CooperativeTile' crates/tiler-compiler/src --include='*.rs'` returned exactly **three** lines at that base, all naming `workgroup_tree_tile`, whose body fixes `rounds: 1` — so every tile any compiler path can build is single-round. A count other than three, or a line naming any other constructor, was the fired verdict *at that base*. **The first form of this check was wrong and is recorded rather than replaced silently**: `grep -rn 'rounds: *[2-9]' crates/ --include='*.rs'` returns three hits, of which two (`crates/tiler-compiler/src/legality.rs`, `crates/tiler-compiler/src/pipeline/conformance.rs`) are an unrelated `rounds` field on a test lowering fixture and one (`crates/tiler-ir/src/schedule/builder.rs`) is a `tiler-ir` schedule test. An emptiness check over that pattern would have read as fired when it is not, and a naming check over the compiler's construction sites is what distinguishes the two. **This CooperativeTile line-count is historical only** (see 2026-08-09 and 2026-08-10).
- 2026-08-09 — **not fired; the old exact line count is no longer authoritative.** Compiler construction still reaches cooperative tiles through `tiler_ir::schedule::workgroup_tree_tile`, whose documented topology remains one round; the additional current mentions are tests, costing, and target checks rather than a second production constructor. `tiler-ir` can still verify a multi-round fixture, but no compiler path builds one, so no new evaluator is required yet. Recheck the construction anchor `let tile = tiler_ir::schedule::workgroup_tree_tile(participants)` in `crates/tiler-compiler/src/physical.rs` and read every other compiler hit before classifying it.
- 2026-08-10 — **not fired.** Production construction still uses `let tile = tiler_ir::schedule::workgroup_tree_tile(participants)` in `crates/tiler-compiler/src/physical.rs`; `fn workgroup_tree_tile` still hard-codes `rounds: 1` (anti-dependency comment in `crates/tiler-ir/src/schedule/cooperative.rs`). The `CooperativeTile` type-name hit count in `tiler-compiler` is no longer the population — hits include measured_cost test helpers, not a second production constructor. `ContributorPartition::covers` still requires an exact product (`Some(total) => total == contributors`). Verified accumulation spend remains empty: `verify_accumulation_width` still enforces equality with region arithmetic, and compiler construction still pins `accumulation: request.numerical_contract().arithmetic`. Multi-round IR fixtures (`fn multi_round_tile_fixture`) still verify with `rounds: 2` without a compiler-built plan.
