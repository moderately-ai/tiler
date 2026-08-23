---
id: lower-and-emit-the-batched-cooperative-contraction
title: Lower and emit the batched cooperative contraction
status: done
priority: p1
dependencies: [admit-a-batched-cooperative-contraction-for-the-attention-structures, honour-the-declared-access-maps-in-the-cooperative-contraction-emission]
related: []
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [contraction, lowering, metal, attention]
---
## User-visible outcome

Both attention structures reach a lowered, emitted cooperative contraction, so the batched form admitted at the schedule layer becomes a plan a target can actually run.

## Why this exists

Split out 2026-08-22 when `admit-a-batched-cooperative-contraction-for-the-attention-structures` **stopped at a gated, coherent boundary**: the schedule layer now admits a rank-N blocked binding with unit batch extents, cover proved at 4,096 threads over 1,600 positions for both structures, and `cooperative_contraction_plan` still refuses rank four **by name** — which is correct at that boundary rather than an oversight.

**Land [`honour-the-declared-access-maps-in-the-cooperative-contraction-emission`](honour-the-declared-access-maps-in-the-cooperative-contraction-emission.md) first.** The emitter currently discards its declared access maps and hardcodes `[M,K]`/`[N,K]`, so a differently-laid-out operand lowers to a silently wrong kernel. The refusal that bounds that population today is exactly the one this ticket removes. Removing it first would make a latent defect reachable.

## What the delivering lane established, and what it warns about

**No new vocabulary is needed and none should be added.** The schedule vocabulary was already sufficient; only the admission predicate was too narrow. Two options were eliminated on correctness rather than cost: a new `ExecutionBinding` variant lands *additively* into wildcards across 22 files in four crates, and a new `ReductionTopology` variant hits `measured_cost.rs`'s undeletable wildcard and collapses `measured_scores` for **every neighbouring alternative**. Re-derive both before reaching for either.

**`tiler.schedule.v7` did not step**, contradicting the parent ticket's own expectation: `push_shape` already frames rank-then-extents, so a rank-four binding was always *encodable* and only ever unadmitted. Do not assume that carries to lowering — derive it.

**Three pieces remain, and the risk is not evenly spread.**

1. A rank-N `cooperative_contraction_plan`.
2. Source-driven addressing — **do not route it through `emit_offset`**, which adds a divide and modulo per operand per round to a kernel with a retained timing.
3. **The rank-N widening of `verify.rs`'s `BlockedGeom`/`IndexRole` abstract interpreter — the highest-risk piece.** It reasons about geometry rather than transporting it, so a rank assumption there fails quietly.

`tiler-metal` needs only a golden: the delivering lane reports zero rank-two hardcoding there and a 1-D dispatch by construction. Verify that rather than inheriting it.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict; all are the delivering lane's and the coordinator verified only the discarded-access-maps site and the participant-rank cap.
- Perturb the subject separately for each new behaviour and quote the failure text. **Before trusting any check, state what it would take for it to say *no* and confirm that case is reachable** — the delivering lane found one of its own tests passed with the clause deleted, because zip truncation refused the fixture on extents instead, and rewrote the fixture to isolate the clause.
- State every identity domain that steps and every one that does not, derived on the merged tree; **stop and report** if one you expect not to move does.
- A Metal golden must be shown to **compile** under the qualified toolchain, and any toolchain fact must name the invocation that produced it — `DEVELOPER_DIR=/Applications/Xcode.app` gives `metalfe-32023.883`, while a bare `xcrun` on this host gives `32023.921`.

  **Correction, 2026-08-22 (worker-lowerbatch).** Both versions are confirmed, but the *source* of the second was imprecise: the bare `xcrun` toolchain is not Xcode-beta. `xcrun --sdk macosx --find metal` resolves to `~/Library/Developer/DVTDownloads/MetalToolchain/mounts/e0303a069097c7034cc5befc63a7a7c2c8ee7720/Metal.xctoolchain/usr/bin/metal`, a downloaded `MetalToolchain` mount, while the pinned invocation resolves to a cryptex mount, `/var/run/com.apple.security.cryptexd/mnt/com.apple.MobileAsset.MetalToolchain-v17.6.109.0.Wgub56/Metal.xctoolchain/usr/bin/metal`. The distinction matters because "switch away from Xcode-beta" is not an available repair for the divergence — neither path is an `Xcode*.app` toolchain, so the two mounts are what a future reconciliation has to name.

## Non-goals

Widening the participant space — `MAX_COOPERATIVE_PARTICIPANT_RANK = 3` sizes the inline arrays behind `ParticipantSpace::new`, so a rank-four participant space is **unrepresentable**, not merely unimplemented; the space stays rank two and the block takes the output's rank. Timing, which needs the bench host. Any new schedule vocabulary.

## Closes when

Both attention structures lower and emit through the batched path, the addressing derives from declared maps rather than an assumed layout, the abstract interpreter is rank-N with its assumptions tested, each behaviour is watched failing on its own subject, identity consequences are derived, and the repository gate is green with the golden compiled under the qualified toolchain.

## Worker record — `worker-lowerbatch`, 2026-08-22

Base `3e6cc78e`; delivered at `75118383` on `tkt/lower-and-emit-the-batched-cooperative-contraction`. Seven files, all inside the declared `implementation/ir` and `implementation/metal`.

### Per-Fact verdict

Every Fact above is the delivering lane's and was re-read at this base.

- **Verified** — `MAX_COOPERATIVE_PARTICIPANT_RANK = 3` (`schedule/mod.rs`), and it sizes the inline arrays behind both `ParticipantSpace::new` and `StagedSpan::new`, each returning `None` above it. A rank-four participant space is unrepresentable.
- **Verified** — `tiler.schedule.v7` did not step for the admission change, and does not step here either: `crates/tiler-ir/src/schedule/` is untouched by this lane (`git diff --stat 3e6cc78e -- crates/tiler-ir/src/schedule/` is empty).
- **Verified** — the schedule layer already admits the batched form. `blocked_batch_prefix` requires every leading block extent to be one, and `verify_blocked_operand_roles` governs the trailing pair alone and names both attention structures.
- **Verified** — `verify.rs`'s `BlockedGeom`/`IndexRole` was the highest-risk piece, and it was load-bearing rather than merely present: reverting `blocked_geometry` to its rank-two-only form refuses both batched structures with `PredicateDominance`.
- **Verified, with a refinement** — `tiler-metal` needed only goldens. Checked rather than inherited: `grep -rnE "extents\(\)\[|extents\(\)\.get\(|\.rank\(\)"` over `crates/tiler-metal/src` and `crates/tiler-metal-aot/src` returns **no matches**, and the dispatch is one-dimensional by construction — `LaunchPlan` carries scalar `grid_threads`/`threads_per_workgroup` and `dispatch.rs` encodes `MTLSize::new(launch.grid_threads, 1, 1)`. The refinement is that "only a golden" is two goldens plus their registration in `GOLDENS`, whose count assertion and directory-listing check both had to move.
- **Imprecise, repaired above** — the bare-`xcrun` toolchain is a downloaded `MetalToolchain` mount, not Xcode-beta.

### Identity consequences, derived

- **`tiler.schedule.v7` — does not step.** No schedule encoder changed; the directory is untouched.
- **`tiler.kernel.v9` — does not step**, on a two-part argument. For rank-two regions the emitted body is byte-identical, which is *directly* evidenced rather than inferred: all thirteen pre-existing Metal goldens are byte-exact pins of emitted text and **none is modified** by this lane — the goldens diff is two additions and zero modifications. For batched regions no body existed before, because `cooperative_contraction_plan` refused them, so there is no prior encoding for a new one to collide with. A new subject taking a new identity under an unchanged domain is what a domain is for; the domain would step only if the meaning of existing bytes changed, and no existing bytes moved.
- **No other domain steps.** `tiler-artifact`, `tiler-build`, and `tiler-compiler` are untouched, so `tiler.kernel-program.v13`, `tiler.index-region.v11`, `tiler.semantic-graph.v3` and the artifact domains are all unaffected.

Nothing moved that was expected not to.

### Perturbations, each on its own subject

Six, each reverted after. Quoted failure text is in the worker report.

1. Dropping the batch term from `blocked_contraction_terms` mis-addresses all four operands (`left: read 261, declared 14341`); the store check and the lowering check still pass.
2. Dropping the batch contribution from the store offset mis-places both stores (`stored at 85, declared 3605`); the operand check still passes.
3. Reading `wg / W_n` as the row workgroup reddens all three batched checks and refuses the lowering with `PredicateDominance`; all 1,323 rank-two tests still pass.
4. Reverting `blocked_geometry` to rank-two-only refuses both structures with `PredicateDominance`, and nothing else.
5. Dropping `axis == geom.row_axis()` from `classify_binary`'s `IndexAdd` is caught by **nothing but** the dedicated unit test — 1,325 other tests pass. This is why the abstract interpreter has direct unit tests: a loose interpreter accepts strictly more, so no canonical body exercises the clause.
6. Making `leading_quotient_is_a_coordinate` the identity reddens the whole rank-two predicated population (13 tests) plus the isolating unit test.

### Gate

`make full` under `DEVELOPER_DIR=/Applications/Xcode.app`, exit 0: 4,044 workspace tests (base 4,029), 1,339 release numerical, 1,343 pinned citations and 7,474 links resolved, `tkt lint` ok, shellcheck ok, both rustdoc passes including `--document-private-items`. Both new goldens compiled and linked under `metalfe-32023.883` — `contraction_batched_tiled_score.metal linked 4563 bytes`, `contraction_batched_tiled_value.metal linked 4563 bytes` — with `TILER_REQUIRE_METAL_TOOLCHAIN=1` set so an absent toolchain would have failed rather than skipped.

### Unsupported cases left standing

A leading block extent above one is still refused, in both `cooperative_contraction_plan` and `blocked_geometry`, because one workgroup would then span several batch coordinates with no participant dimension to distinguish them. A second contracted coordinate is still refused: the round loop walks one induction. An output position past the trailing pair is still refused. Square tiles only. No timing was taken — that needs the bench host.
