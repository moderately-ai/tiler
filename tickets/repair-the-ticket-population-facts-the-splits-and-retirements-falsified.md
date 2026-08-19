---
id: repair-the-ticket-population-facts-the-splits-and-retirements-falsified
title: Repair the ticket-population facts the splits and retirements falsified
status: in-progress
priority: p1
dependencies: []
related: [repair-the-accepted-decision-records-the-splits-and-retirements-falsified, repair-the-navigation-and-contract-docs-the-audit-falsified, repair-the-research-records-the-key-replacement-and-splits-falsified, emit-the-indirect-gather-on-metal]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, audit, tickets]
claimed_from: todo
assignee: worker-tickets
lease_expires_at: 1787162948
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

## Per-Fact verdict — 2026-08-19, base `f08281a1`

Every Fact in this ticket re-audited against source before any edit. **This ticket's own Facts carried three errors, two of which would have propagated into the repairs.**

| # | Fact as stated | verdict | evidence |
| --- | --- | --- | --- |
| 1 | `StorageScalar` has four variants incl. `U32` | **verified** | `crates/tiler-ir/src/program/model.rs`, anchor `pub enum StorageScalar`; `U8`, `F32`, `Bf16`, `U32`, the last doc'd `An unsigned 32-bit integer carrier` |
| 2 | `widen-the-physical-vocabulary…` carries the same stale premise | **verified, and understated** | anchor `naming truthful` resolves; the *same sentence* also miscounts `KernelType` as five — it has seven (`Bool`, `U8`, `Index`, `F32`, `I32`, `Bf16`, `U32`) |
| 3 | `crates/tiler-ir/src/schedule/builder.rs` does not exist | **verified** | `ls` → `No such file or directory` |
| 4 | the replacing directory holds the 12 listed files | **FALSE** | the directory holds **13**; the listing omits `tile.rs` |
| 5 | `verify_cooperative_tile` → `builder/tile.rs` *(flagged unverified)* | **verified — the flag was wrong, not the row** | `crates/tiler-ir/src/schedule/builder/tile.rs`, anchor `pub(super) fn verify_cooperative_tile`. The row was doubted **because of** Fact 4's defective listing |
| 6 | other four relocation rows | **verified** | `reads_bind_boundary_tensors_in_order`→`builder/elementwise.rs`; `verify_intrinsic`→`builder/intrinsic.rs`; `verify_accumulation_width`→`builder/reduction.rs`; `multi_round_tile_fixture`→`builder/tests.rs`; `split_family`→`builder/family.rs` |
| 7 | the `rg … builder.rs` command errors | **verified, and understates the class** | exit 2, `No such file or directory (os error 2)`. Two further commands in the same file use brace expansion, exit 2 **while still printing partial results** — a worse shape, since they look like they worked |
| 8 | census is "roughly sixty-five" files | **imprecise** | 68 by the literal pattern, 63 brace-aware; partition below |
| 9 | `package-the-admitted…` cites two relocated symbols | **verified** | → `request/tests.rs` and `program/tests/baked_extents.rs`; `request.rs` still exists (resolves to a live wrong file), `program/tests.rs` does not |
| 10 | **four open** tickets cite the deleted contraction constants | **FALSE** | three of the four are `done` — `bound-the-reference-contraction-comparison-for-the-profile-cells`, `decide-the-semantic-order-contract-for-relaxed-contractions`, `decide-the-algebraic-capability-authority-for-contraction-splits`. Only `admit-reassociated-contraction-schedule-alternatives` is open, and it **already** records the correction. Net repairs from this Fact: **zero** |
| 11 | the constants exist nowhere in `crates/` | **verified** | 0 hits; `BF16_FACT_REASSOCIATION_PERMITTED` is a live *different* constant and must not be confused with them |
| 12 | two deferred tickets name the retired key | **verified** | `scope-the-windowed-reduction-and-convolution-family`, `research-an-explicit-seeded-fused-contraction-operation`; successor `tiler::tensor-contraction-f32@1` |

**Fact 4 is the load-bearing error.** The omission of `tile.rs` is what caused row 5 to be flagged unverified, and the brief instructed the worker to contradict it — which is the correct outcome, but the mechanism is worth recording: an incomplete enumeration presented as complete manufactured a false doubt about a true row. Had the worker deferred to the flag, the deferred ticket citing `verify_cooperative_tile` would have been left unrepaired.

**Fact 10 is the error that would have wasted the most work**, and it is the failure mode AGENTS.md names directly: the instruction "enumerate rather than sample" was attached to a list that had itself not been filtered by ticket state. Three of its four entries are history by the same convention this ticket applies elsewhere.

### Facts found beyond those stated

- **Stale identity domains in two open implementation tickets.** Both materialization-edge siblings pin `tiler.schedule.v5`; one also pins "current `tiler.kernel.v7`". At this base they are `tiler.schedule.v7` and `tiler.kernel.v9`. Repaired by naming the domains rather than fresh numbers — pinning a number is what rotted the originals.
- **A wider encoder population than either ticket states.** Both say schedule and kernel `push_tensor_role` write a bare `Intermediate` tag. **Four** encoders do: `tiler-ir/src/schedule/model.rs`, `tiler-ir/src/kernel/model.rs`, `tiler-compiler/src/selection.rs`, `tiler-compiler/src/frontier.rs`. The accepted Option A migration is larger than the sentence implied.
- **Unverified and left marked as such:** the "ADR 0074 5b total maps in three compiler files" count in both siblings. 11 files under `crates/tiler-compiler/src/` name `TensorRole::Intermediate`; distinguishing total maps from mentions needs a full read of each and was out of this lane's scope.
- **Two supplied commands run but return different results than claimed.** `scope-the-windowed-reduction…` records 46 governed keys; the command returns **50**. `derive-the-exact-evaluator…`'s 2026-08-05 log records three `rounds: *[2-9]` hits; there are now **five**. Neither changes its entry's conclusion; both recorded, both dated entries left verbatim as evidence about their own base.
- **A stale-state finding one level below the gather ticket**, reported not acted on: `admit-the-selected-data-dependent-index-representation` is `blocked` while both of its declared dependencies are `done`.

## Closing census — 2026-08-19, base `f08281a1`

**The census pattern this ticket was filed with undercounts, and that is the first finding.** A grep for the literal `schedule/builder.rs` misses shell brace expansions such as `crates/tiler-ir/src/schedule/{model.rs,builder.rs}`, which name the deleted file while containing no matching substring. `admit-reassociated-contraction-schedule-alternatives` carries two such citations beside its one literal one, so the literal sweep reported it as one stale site when it has three. Both commands are recorded below.

Literal population, partitioned by ticket state:

```sh
grep -rl 'schedule/builder\.rs' tickets/ | wc -l          # 68
```

| state | count | treatment |
| --- | --- | --- |
| `done` | 57 | history by repository convention — **not repaired** |
| `closed` | 1 | history — **not repaired** |
| `todo` | 3 | repaired |
| `deferred` | 3 | repaired (a deferred ticket is dispatchable later) |
| `blocked` | 1 | repaired |
| `in-progress` | 3 | this lane's own file plus two sibling repair lanes — **not touched**, per Coordination below |
| **total** | **68** | 58 excluded as history, 7 repaired, 3 lane files |

Brace-aware population, which is the authoritative one:

```sh
grep -rn 'builder\.rs' tickets/ \
  | grep -E 'tiler-ir/src/schedule/(\{[^}]*)?builder\.rs|schedule/\{[^}]*builder\.rs'
```

This yields **63 files**, with the same state partition and no additional open ticket beyond those already named.

**Neither count can be used as a "remaining stale citations" metric, and re-running them after this lane will mislead.** Every repair here follows the repository's dated-correction convention, which quotes the retired path verbatim (`crates/tiler-ir/src/schedule/builder.rs no longer exists`) so the correction is itself searchable. A repaired file therefore still matches both patterns. Measured directly: the literal count moved only 68 → 67 across seven repaired files, and the `deferred` partition did not move at all. This is the inverse hazard AGENTS.md names — a hit is evidence the string is present, not that the claim still stands — and it means **a closing condition demanding either grep be empty is unmeetable by construction.** The counts above are the pre-repair population and are stated as such; the repair evidence is the per-file dated corrections, not a shrinking grep. A wider `grep -rl 'builder\.rs' tickets/` matches **134** files, but the excess names *live* `builder.rs` files that the splits did not touch — `crates/tiler-ir/src/kernel/builder.rs`, `crates/tiler-ir/src/index/builder.rs`, `crates/tiler-ir/src/program/builder.rs`, and `crates/tiler-artifact/src/program/builder.rs` all exist at this base. Four open tickets appear only in that wider set and were checked and excluded on that ground: `admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir`, `deliver-several-artifact-families-from-one-expansion`, `package-selected-physical-implementation-provenance-in-artifact-identity`, and `replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges`.

### Site added by the coordinator mid-lane, outside the builder population

`split-metal-profile-measurement-sources-by-compilation-selection` (`todo`, p1) instructs a worker to update `nonprojected_metal_facts_do_not_reach_the_compiler_descriptor`. **Verified deleted:** 0 hits in `crates/`, and `nonprojected` as a term also returns 0, so the concept was retired rather than renamed. Removed by `1f6ec214`, an ancestor of this base and the landing of that ticket's own `done` dependency. Repaired in place with the obligation restated over the three tests that replaced it; closure **not** claimed, since this lane holds neither `implementation/build` nor `implementation/compiler`. The coordinator's exclusion of `construct-and-bind-the-first-authoritative-metal-compile-profile` was independently confirmed — it is `status: done`, so both its stale test name and its "exactly two overlaps" Fact are history here.

**One open ticket in the literal population was deliberately not repaired.** `keep-a-module-size-and-complexity-census-with-a-split-queue` (`todo`) cites `tiler-ir/src/schedule/builder.rs` at 10,554 lines inside a section headed `Census — 2026-08-19, base e53eb65f working tree`. That is a dated measurement of a pre-split tree, and its own file records the resolution below it — the `First tranche landed` section names the builder split's merge commit `56d95195`. Repairing the row would falsify the measurement it exists to preserve, so it is left and reported instead.

## Coordination

`project/tickets` is declared **shared** rather than exclusive so this lane can run beside its three sibling document tickets, which each append to their own ticket file under the repository's default shared declaration. That is safe only because the file sets are disjoint, and keeping them disjoint is this lane's obligation: **do not edit `repair-the-accepted-decision-records-the-splits-and-retirements-falsified.md`, `repair-the-navigation-and-contract-docs-the-audit-falsified.md`, `repair-the-research-records-the-key-replacement-and-splits-falsified.md`, `size-the-numerical-realization-flag-list-from-its-type.md`, or `re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement.md`.** Report the exact file list you touched so the coordinator can confirm disjointness against the sibling diffs before merging.

## Non-goals

`docs/**` and any source change — three sibling tickets hold those scopes. Do not unblock, close, or re-prioritize any ticket yourself, and do not re-litigate ADR 0112 or the `U32` carrier's admission.

## Closes when

Every open ticket's stale Fact above is repaired or verified already-correct with evidence, every supplied command in a touched ticket has been rerun and reported, the `tickets/` census is quoted with its state partition and counts, the gather ticket's readiness determination is written down, and `tkt lint` plus `make citations` are green.
