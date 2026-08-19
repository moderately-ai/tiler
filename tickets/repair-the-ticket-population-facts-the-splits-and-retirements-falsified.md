---
id: repair-the-ticket-population-facts-the-splits-and-retirements-falsified
title: Repair the ticket-population facts the splits and retirements falsified
status: todo
priority: p1
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified, repair-the-navigation-and-contract-docs-the-audit-falsified, repair-the-research-records-the-key-replacement-and-splits-falsified, emit-the-indirect-gather-on-metal]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, audit, tickets]
---
## User-visible outcome

No open ticket routes a worker to a deleted file, supplies a command that errors instead of returning a result, or rests its blocked state on a Fact the tree has since falsified. At least one ticket's blocking premise has already lifted and its state is corrected accordingly.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and re-verified by the coordinator at `de18ebdb`. This is p1 above its three sibling document tickets because a false ticket Fact is the input to *future work*, not only to a reader — AGENTS.md's stale-Facts rule exists because a worker who trusts one produces a wrong diff.

**This population was missed twice before, both times by a Closes-when that scoped `docs/` and not `tickets/`.** `point-the-bare-builder-path-mentions-at-the-split-modules` is `done` on `grep -rln "schedule/builder\.rs" docs/`; `re-anchor-the-schedule-builder-line-citations` is `done`; `execute-the-doc-drift-sweep-the-audit-enumerated` is `done`. None of their closing censuses read `tickets/`. **State the population in this ticket's own closing census and include `tickets/`.**

**Fact — a blocked ticket's blocking premise has lifted.** `tickets/emit-the-indirect-gather-on-metal.md` is `status: blocked`, and its **Fact — no integer storage carrier** asserts that `pub enum StorageScalar` "currently has three variants, `U8`, `F32`, and `Bf16`". At this base it has **four**: `U8`, `F32`, `Bf16`, and `U32`, the last documented "An unsigned 32-bit integer carrier" and giving its natural access type as the exact-width `KernelType::U32` (`crates/tiler-ir/src/program/model.rs`, anchor `pub enum StorageScalar`). This is scheduling-relevant, not only prose: the stated reason the ticket is blocked no longer holds. **Do not simply unblock it** — read what else its dependency edges rest on, repair the Fact, and report whether the ticket is now ready, still blocked for a different stated reason, or needs its dependencies repointed. That determination is this ticket's deliverable; the coordinator makes the state change.

`tickets/widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md` (`todo`) carries the same stale premise at anchor `naming truthful \`U8\` and \`F32\` carriers`.

**Fact — six or more open tickets cite a deleted module in load-bearing Facts, and one supplies a command that cannot run.** `crates/tiler-ir/src/schedule/builder.rs` does not exist; the directory `crates/tiler-ir/src/schedule/builder/` replaced it (`contraction.rs`, `copy.rs`, `coverage.rs`, `diagnostics.rs`, `elementwise.rs`, `family.rs`, `intrinsic.rs`, `mod.rs`, `proof.rs`, `reduction.rs`, `structural_relation_tests.rs`, `tests.rs`). Confirmed relocations, so the worker need not re-derive them:

| ticket | status | cited symbol → verified location |
| --- | --- | --- |
| `admit-a-scheduled-region-that-reads-two-materialization-edges` | blocked | `reads_bind_boundary_tensors_in_order` → `builder/elementwise.rs` |
| `admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region` | todo | same |
| `admit-a-round-dependent-cooperative-staging-span` | deferred | `verify_cooperative_tile` → `builder/tile.rs` *(unverified — this ticket's coordinator could not confirm `builder/tile.rs` exists in the listing above; locate the symbol before repointing)* |
| `realize-the-tiled-contraction-schedule-and-its-metal-emission` | deferred | `verify_intrinsic` → `builder/intrinsic.rs` |
| `derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order` | deferred | `verify_accumulation_width` → `builder/reduction.rs`; `multi_round_tile_fixture` → `builder/tests.rs` |
| `admit-reassociated-contraction-schedule-alternatives` | todo | `split_family` → `builder/family.rs` |

The last supplies a reproducing command that errors rather than returning a result: `rg -n 'multi_pass_family|cooperative_family|fn split_family|StrictTensorContraction' crates/tiler-ir/src/schedule/builder.rs` → `No such file or directory (os error 2)`. **A supplied command that has never been executed is a claim, not a check** — rerun every command this sweep touches and repair the ones that do not run.

The full `tickets/` census at this base is far wider than the six rows above — roughly sixty-five ticket files contain the string. Many are `done` or `closed`, where the citation is history by repository convention and is **not** repaired. Partition the census by ticket state and repair only what a future worker can be dispatched from; report both counts so the boundary is visible.

**Fact — `package-the-admitted-live-schedule-into-a-symbolic-kernel-program` (todo) cites two relocated symbols inside its recommended decision option**, which is the highest-consequence place in that file for a stale citation: `a_compiled_plan_does_not_fold_a_bound_extent_value` → `crates/tiler-compiler/src/request/tests.rs`, and `baking_neighbouring_extents_mints_distinct_artifact_subjects` → `crates/tiler-artifact/src/program/tests/baked_extents.rs`. Note `crates/tiler-compiler/src/request.rs` **still exists** as the spine beside its new `request/` submodules, so the path is not dangling even where the symbol has moved — treat path and symbol separately throughout.

**Fact — four open tickets cite the two deleted contraction-fact constants.** `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` and `CONTRACTION_F32_FACT_PERMUTATION_PERMITTED` do not exist anywhere in `crates/` at this base; `reduction_descriptor_record` (`crates/tiler-ir/src/semantic/contraction.rs`) declares the row `"permission-gated"` instead. Cited in `bound-the-reference-contraction-comparison-for-the-profile-cells`, `decide-the-semantic-order-contract-for-relaxed-contractions`, `decide-the-algebraic-capability-authority-for-contraction-splits`, and `admit-reassociated-contraction-schedule-alternatives`. **The audit that surfaced this class named only one document and none of these four tickets** — enumerate rather than sample.

**Fact — two deferred tickets name the retired contraction key as the current standard identity.** `scope-the-windowed-reduction-and-convolution-family` and `research-an-explicit-seeded-fused-contraction-operation`, against ADR 0112 and the pin in `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs`. A deferred ticket is still dispatchable later, so its Facts are repaired, not excused.

## Required work

- Re-audit every Fact above at your actual base and report a per-Fact verdict before editing. The table's `builder/tile.rs` row is explicitly flagged unverified — contradict it with evidence rather than propagating it.
- Repair each open ticket's stale Fact in place with a dated correction, preserving what remains true. Do not silently restate a false Fact in new words, and do not repair a Fact by deleting the reasoning it supported.
- For every citation, locate the named **symbol** and cite by searchable anchor rather than by path alone or by line number. Run each anchor's grep against the file its citation names before writing it.
- Rerun every reproducing command in a ticket you touch. Repair the ones that error; report any that run but return a different result than the ticket claims.
- Produce the closing census over `tickets/` **and** state the state partition (which files were repaired, which are `done`/`closed` history left alone) with counts.
- For `emit-the-indirect-gather-on-metal`, deliver the readiness determination described above as a written finding; leave the state change to the coordinator.

## Coordination

`project/tickets` is declared **shared** rather than exclusive so this lane can run beside its three sibling document tickets, which each append to their own ticket file under the repository's default shared declaration. That is safe only because the file sets are disjoint, and keeping them disjoint is this lane's obligation: **do not edit `repair-the-accepted-decision-records-the-splits-and-retirements-falsified.md`, `repair-the-navigation-and-contract-docs-the-audit-falsified.md`, `repair-the-research-records-the-key-replacement-and-splits-falsified.md`, `size-the-numerical-realization-flag-list-from-its-type.md`, or `re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement.md`.** Report the exact file list you touched so the coordinator can confirm disjointness against the sibling diffs before merging.

## Non-goals

`docs/**` and any source change — three sibling tickets hold those scopes. Do not unblock, close, or re-prioritize any ticket yourself, and do not re-litigate ADR 0112 or the `U32` carrier's admission.

## Closes when

Every open ticket's stale Fact above is repaired or verified already-correct with evidence, every supplied command in a touched ticket has been rerun and reported, the `tickets/` census is quoted with its state partition and counts, the gather ticket's readiness determination is written down, and `tkt lint` plus `make citations` are green.
