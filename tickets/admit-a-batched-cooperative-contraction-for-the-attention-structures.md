---
id: admit-a-batched-cooperative-contraction-for-the-attention-structures
title: Admit a batched cooperative contraction for the attention structures
status: todo
priority: p1
dependencies: []
related: [realize-the-attention-contractions-on-metal, realize-the-tiled-contraction-schedule-and-its-metal-emission, offer-the-tiled-contraction-alternative-once-a-width-authority-exists]
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, contraction, attention, public-boundary, needs-tom]
---
## User-visible outcome

The tiled cooperative contraction can realize a contraction whose output carries batch axes, so the two attention structures become reachable by a realization other than `direct`.

## Why this exists

Filed 2026-08-22 by `worker-attention` from [`realize-the-attention-contractions-on-metal`](realize-the-attention-contractions-on-metal.md), which found this while re-deriving the schedule set from the current synchronization vocabulary. **It is a vocabulary gap, not a precondition failure, and the two must not be conflated.**

**Fact — both attention structures produce a rank-four output.** `grtd,gsd->grts` gives `[g, r, t, s]` and `grts,gsd->grtd` gives `[g, r, t, d]`.

**Fact — the cooperative contraction is rank-two-output at three independent layers**, each verified by reading the file at base `1fb3675c`:

- `crates/tiler-ir/src/schedule/cooperative.rs`, `blocked_operand_tile`, builds a rank-two participant space and states the choice at anchor `Rank two, deliberately`.
- `crates/tiler-ir/src/schedule/builder/contraction.rs` couples that space to the binding block through `participant_space_matches_block`, which compares ranks.
- `crates/tiler-ir/src/kernel/lower.rs`, `cooperative_contraction_plan`, refuses any region whose `iteration_shape.rank() != 2` with `KernelDiagnostic::CooperativeLoweringShape`.

**Fact — the refusal is already typed and already watched firing.** `crates/tiler-ir/tests/attention_tiled_admission.rs`, anchor `the_accepted_blocked_tile_cannot_cover_a_rank_four_attention_output`, asserts the exact `OutputBlockRankMismatch { output_rank: 4, block_rank: 2 }`. So nothing here is silently wrong today; the realization is withheld by name.

**Inference — this is why "`tiled` for both structures" was unmet and could not be met by choosing a width or an extent.** The contracted-extent precondition is a property of one row's `S` and disappears at `S = 128`; the rank wall holds at every row of both structures, including the admissible ones.

## Why the deleted-guard experiment is the argument for care

Deleting the rank guard from `ceiling_quotients` — done and reverted by the filing lane — made the authority return `PredicatedCooperativeContraction { ... work_items: 1600, grid_threads: 256 }` for structure 2's C1 prefill output. That is a launch covering 256 of 1,600 output positions, admitted as proven. A batched form must therefore extend the *cover proof*, not relax the rank check.

## Required work

- Re-audit every Fact above at your own base and report a per-Fact verdict before editing.
- Decide and state whether batch axes are carried as block extents of one on a widened rank-N blocked binding, or as a separate declared batch prefix. These are materially different identities; enumerate both plus the status quo under the decision-packet readiness gate.
- Preserve `prove_blocked_bijection` and `prove_blocked_predicated_cover` as **cover proofs**: every logical output coordinate keeps exactly one launched preimage. A widened rank must strengthen the theorem, never weaken it.
- State every identity domain that steps. A widened `ExecutionBinding` or participant space is a schedule-encoder change and is expected to move `tiler.schedule`; derive it rather than assuming.
- Perturb each new behaviour separately, subject not assertion, with quoted failure text.

## Non-goals

Offering the alternative in physical planning, which is [`offer-the-tiled-contraction-alternative-once-a-width-authority-exists`](offer-the-tiled-contraction-alternative-once-a-width-authority-exists.md)'s and needs a width authority; choosing a tile width; and the `direct` realization, which already covers both structures.

## Closes when

A contraction with batch axes verifies, lowers, and emits under the cooperative topology with its cover proof strengthened rather than relaxed, the rank refusal still fires for genuinely unsupported shapes, and the identity consequences are derived and recorded.

## Independent derivation — flattening the output is not an escape

Added 2026-08-22 by `worker-attention` after the rank wall was found, because the obvious cheap answer needs closing off before anyone spends a day on it.

**The tempting shortcut.** If the cooperative contraction only accepts a rank-two output, express the attention result `[g, r, t, s]` as `[g * r * t, s]` and hand it the flattened form. No new vocabulary, no widened proof.

**Fact — it is inexpressible, and the reason is the access map rather than the shape.** `ContractionAxisSource` in `crates/tiler-ir/src/schedule/model.rs` has exactly two variants, `Output { position }` and `Contracted { position }`, each naming a **whole axis position** in the output or contracted shape. There is no variant naming a sub-range, a stride, or a quotient of an axis.

**Inference — so the key operand cannot be addressed against a flattened output.** The score structure's key operand is `[g, s, d]`: it reads `g` and `s` and never `r` or `t`. Against `[g * r * t, s]` the coordinate `g` is `flat / (r * t)`, which is not an axis of that shape and therefore not nameable by `Output { position }`. The same holds for the value structure's `[g, s, d]`. Flattening would force the key operand to be materialized across `r` and `t` — which is precisely the broadcast the grouped-query structure exists to avoid, and which `the_score_kernels_key_address_is_independent_of_the_repetition_index` in `crates/tiler-metal/src/tests.rs` now refuses by name.

**Consequence.** The batch axes must be carried as batch axes. This is a genuine vocabulary extension, and the option enumeration this ticket requires should record flattening as eliminated rather than omit it.

## Source-first Fact audit — 2026-08-22, exact base `4f53343ffd6732db1a7f9828aca3cad9c9a9ea06`

Every Fact re-read in the file it names, at this base, by `worker-batched`.

| # | Fact as stated | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Both attention structures produce a rank-four output | **verified** | `crates/tiler-ir/tests/attention_tiled_admission.rs` asserts `output.rank() == 4` for `[g, r, t, s]` and `[g, r, t, d]` |
| 2a | `blocked_operand_tile` builds a rank-two participant space, anchor `Rank two, deliberately` | **verified** | `crates/tiler-ir/src/schedule/cooperative.rs`; the anchor resolves, and the constructor calls `ParticipantSpace::new(&[block, block])` |
| 2b | `participant_space_matches_block` couples that space to the binding block by comparing ranks | **verified at base** | `crates/tiler-ir/src/schedule/blocked.rs`, called from `verify_cooperative_contraction`. Deliberately falsified by this branch — see below |
| 2c | `cooperative_contraction_plan` refuses any region whose `iteration_shape.rank() != 2` | **verified but imprecise** | the guard is compound: it also fixes `block.rank()`, `workgroups.rank()`, `contracted_shape.rank()`, `contracted_tile.rank()`, **and** requires `block_m == block_n == tile_k`. The wall is wider than "output rank two", and a repair that only widened the output rank would still be refused |
| 3 | The refusal is typed and already watched firing | **verified** | `the_accepted_blocked_tile_cannot_cover_a_rank_four_attention_output` asserts the exact `OutputBlockRankMismatch { output_rank: 4, block_rank: 2 }` |
| 4 | Flattening is inexpressible: `ContractionAxisSource` has exactly two whole-axis variants | **verified** | `crates/tiler-ir/src/schedule/model.rs`, `Output { position }` and `Contracted { position }`; no sub-range, stride, or quotient variant. The premise of this ticket stands |

### Three facts the ticket does not contain, each of which changed the design

**Fact — the cover proofs were already rank-general, so nothing needed widening there.** `admit_exact_cooperative_contraction`, `admit_predicated_cooperative_contraction`, `prove_blocked_bijection`, and `prove_blocked_predicated_cover` in `crates/tiler-ir/src/schedule/blocked.rs` all compare ranks and then iterate per axis. The ticket's Required-work item "a widened rank must strengthen the theorem" is discharged by *using* these functions at rank four rather than by editing them; they are untouched on this branch.

**Fact — the ticket's first option cannot be built the way it is worded.** `MAX_COOPERATIVE_PARTICIPANT_RANK` is `3` (`crates/tiler-ir/src/schedule/mod.rs`), and it sizes the inline arrays inside `ParticipantSpace` and `StagedSpan`, so `ParticipantSpace::new` returns `None` above it. A rank-four participant space is **unrepresentable, not merely unimplemented**. Batch axes therefore cannot be carried as participant dimensions of a widened rank-N space; the space stays rank two and the *block* takes the output's rank.

**Fact — the emission discards the declared access maps.** `emit_cooperative_contraction` in `crates/tiler-ir/src/kernel/lower.rs` contains `let _ = (left_addr, right_addr);` and hardcodes `row * contracted + k` / `col * contracted + k`, i.e. an `[M, K]` × `[N, K]` layout. Nothing in the schedule verifier or the kernel verifier constrained the operand sources to that layout, so a region declaring its left operand as `[K, M]` verified and lowered to a silently wrong kernel. **This is directly on the batched path rather than an unrelated defect**: the value structure `grts,gsd->grtd` has a right operand `[g, s, d]` whose contracted axis sits in the *middle*, so it is a `[K, N]` layout that the hardcoded addressing computes incorrectly. Source-driven addressing is a prerequisite of the batched lowering, not polish.

## Decision — batch axes are carried as unit block extents on a rank-N blocked binding

Taken rather than escalated, under the decision-packet readiness gate's own rule that a dominant option is taken rather than turned into a question. The materially distinct options, with the two eliminations stated before ranking:

| Option | Verdict |
| --- | --- |
| **A. Rank-N block, batch axes of extent one, participant space stays rank two** | **taken** |
| B. A separate declared batch prefix on a new `ExecutionBinding` variant | **eliminated** |
| C. Flatten the output to `[g*r*t, s]` | **eliminated** — the ticket's own independent derivation, re-verified as Fact 4 |
| D. Status quo — `direct` only for both structures | **eliminated** — it is the condition this ticket exists to remove, and it leaves a real defect (the discarded access maps) standing |
| E. A new `ReductionTopology` variant, e.g. `BatchedCooperativeContraction` | **eliminated** |

**B is eliminated on correctness, not on effort.** `ExecutionBinding` is `#[non_exhaustive]`, so a new variant lands *additively* in every out-of-crate consumer's wildcard rather than breaking it. `ExecutionBinding::` is matched in 22 files across `tiler-build`, `tiler-artifact`, `tiler-compiler`, and `tiler-conformance` — every one outside this ticket's scopes — and a batched binding silently absorbed by one of those wildcards is precisely the failure mode this repository names. It also duplicates a cover theorem that `blocked.rs` already proves rank-generally.

**E is eliminated for the reason the brief names.** `ReductionTopology` is `#[non_exhaustive]` outside `tiler-ir`, and `work_span` in `crates/tiler-compiler/src/measured_cost.rs` carries a wildcard arm it cannot delete (`E0004`). A new variant would decline there, `assess_fold_steps` propagates the decline with `?`, and `measured_scores` collects `Option<Vec<_>>` over every retained alternative — so one new topology would collapse measured selection for its neighbours, which the retained sweep measured at up to 50.7x slower. Option A adds no variant, and `work_span`'s existing `CooperativeContraction` arm is already rank-agnostic (it reads `element_count(contracted_tile)`, `tile.rounds`, and `work_items`), so measured selection keeps working with **no edit to that out-of-scope file**.

**Why A dominates on the remaining dimensions.** It adds no tag, no variant, no encoder byte, and no public surface; the block and workgroup shapes are already rank-framed by `push_shape`, so a rank-four binding was always *encodable* and only ever unadmitted. It confines the change to `tiler-ir` and `tiler-metal`, which is this ticket's exact scope set — a signal that this is the intended factoring rather than a coincidence.

## Delivered — the schedule layer admits the batched form, with its cover proved

### The rule, and why it strengthens rather than relaxes the theorem

`participant_space_matches_block` no longer compares ranks. It now requires the participants to occupy the block's **trailing two axes** and every leading block extent to be one. At rank two the prefix is empty and the rule is the exact equality it always was, which is why the batched rule *replaces* the old one rather than sitting beside it — a rank-two block is the batched block with no batch axes, and two predicates would be two places to state one relation.

The soundness argument is a bijection, not a weakening: a leading extent of one admits exactly one block-local coordinate, zero, so the map from participant `(l_0, l_1)` to block-local position `(0, .., 0, l_0, l_1)` is still onto every position the block contains and still injective. The participant count therefore still equals the block's element count, which is the equality the two cover proofs compose against the launch geometry.

`verify_blocked_operand_roles` is new and states what the staged tile always required and never checked: the left operand reads the block's row axis and not its column axis, and the right the reverse. Batch axes are deliberately unconstrained, which is what keeps the grouped-query structure expressible — the key operand `[g, s, d]` shares the group and never reads the repetition — and keeps the ordinary batched matmul, where both operands read a shared batch axis, admitted.

### The cover, measured rather than asserted

`a_batched_block_covers_every_output_position_of_both_attention_structures` admits a rank-four block `[1, 1, 16, 16]` for both structures and then runs `prove_blocked_predicated_cover` over the result. Observed on this branch:

| Structure | output | workgroups | `work_items` | `grid_threads` | rounds |
| --- | --- | --- | --- | --- | --- |
| score `grtd,gsd->grts`, C1 prefill | `[8, 2, 10, 10]` | `[8, 2, 1, 1]` | 1,600 | 4,096 | 8 |
| value `grts,gsd->grtd`, `S = 128` | `[8, 2, 10, 128]` | `[8, 2, 1, 8]` | 20,480 | 32,768 | 8 |

**This is the direct answer to the deleted-guard experiment.** Deleting the rank guard produced `work_items: 1600, grid_threads: 256` — a launch covering 256 of 1,600 positions, admitted as proven. Carrying the batch axes *as batch axes* produces 4,096 against 1,600: a cover with an inactive remainder, which `prove_blocked_predicated_cover` accepts and which it would refuse at 256 on `MappingGap`.

**The existing refusal is intact.** A rank-two block against a rank-four output still returns `OutputBlockRankMismatch { output_rank: 4, block_rank: 2 }`, verified on this branch, because that pairing is a genuine mismatch rather than a batched block. `the_accepted_blocked_tile_cannot_cover_a_rank_four_attention_output` is unchanged and still passes.

### Perturbations — subject broken, not assertion, each clause separately

| Guard | Perturbation | What it said |
| --- | --- | --- |
| the unit-batch-extent clause | `blocked_batch_prefix` made to return `Some(prefix)` unconditionally | `a_batch_axis_wider_than_one_workgroup_is_refused` — `assertion left == right failed / left: Some(2) / right: None`; the other five in the module stayed green |
| the participant-rank clause | the `participants.rank() == 2` conjunct deleted outright | `a_participant_space_of_another_rank_is_refused` — `the batch axis carries no participant dimension, so a space of the block's own rank is not the space this tile states`; the other five stayed green |
| the left-operand role | the `left_reads[column]` conjunct deleted | `a_left_operand_reading_the_column_axis_is_refused` — `left: Ok(()) / right: Err(BlockedWorkgroup { rule: ParticipantBlockMismatch })`; the other six stayed green |
| the right-operand role | the `right_reads[row]` conjunct deleted | `a_right_operand_reading_the_row_axis_is_refused` — same text, one test red, the other six green |

**A perturbation refuted one of my own tests, and it was repaired rather than dropped.** `a_participant_space_of_another_rank_is_refused` was first written with a rank-three space `[1, 16, 16]` against block `[1, 16, 16]`. Deleting the rank clause left it **passing**, because the zip against the block's trailing `[16, 16]` truncates and the leading `1 != 16` comparison refuses it on extents instead. The case could not tell the two refusals apart. It now uses `[16, 16, 4]`, which zips equal on the compared prefix and differs only in carrying a third dimension, and it asserts the participant count is 1,024 against the block's 256 so the rank clause cannot be re-derived from the extents. The test's own doc comment records this.

Each perturbation reddens exactly one test and leaves its siblings green, which is what shows the clauses are independently load-bearing rather than one predicate failing four ways.

## Identity consequences — derived on this tree, not copied

| Domain | Moves? | Derivation |
| --- | --- | --- |
| `tiler.schedule.v7` | **no** | No encoder byte was touched. `push_schedule` is unchanged; no `ReductionTopology` or `ExecutionBinding` variant, tag, or field was added or reordered. `push_shape` frames rank then extents, so a rank-four `block`/`workgroups` was *already* encodable — this branch changes which such bindings are **admitted**, not how any of them encodes. Reduction-topology tags stay `0x31`-`0x35`, `0x37`, `0x38`; **`0x36` was not taken** and stays reserved for `CooperativeContractionSplit`; `0x39` stays unconsumed |
| `tiler.kernel.v9`, `tiler.kernel-program.v13` | **no** | Nothing under `crates/tiler-ir/src/kernel/` was edited. The lowering is byte-identical |
| `tiler.artifact-program.v22`, manifest pair | **no** | The artifact layer was not edited |
| Every Metal golden, all 13 | **no** | `crates/tiler-metal` is untouched. `contraction_tiled_cooperative.metal` keeps kernel digest `f4bec9ec1d6a846e` and region digest `2ecefc369c138064`, confirmed by the workspace suite passing; `GOLDENS` stays at 13 entries |
| Target-profile declaration/descriptor `v11` | **no** | No profile row, policy family, or key was added; no tile-width authority was declared |
| Public surface | **no additions** | `participant_space_matches_block` is `pub` inside `mod blocked`, which is **private** and re-exports it nowhere — verified against the `pub use blocked::{…}` list in `crates/tiler-ir/src/schedule/mod.rs`. It is crate-internal, so widening it is an internal-API change and not a boundary decision. `blocked_batch_prefix` and `verify_blocked_operand_roles` are both private. The only new re-export used is `prove_blocked_predicated_cover`, which was already public |

**Nothing that was expected not to move, moved.** The one domain a reader would reasonably expect to step — `tiler.schedule`, since the ticket names a widened binding as a schedule-encoder change — provably does not, and the derivation above is why: the widening is in the *admission predicate*, and the encoder was already rank-general. That is a genuine divergence from the ticket's own expectation, stated rather than glossed.

## Remainder — the lowering and the emission, mapped with a complete design

**This branch stops at a gated boundary and does not meet this ticket's Closes-when.** A batched contraction now *verifies*; it does not yet lower or emit, so `cooperative_contraction_plan` still refuses it by `CooperativeLoweringShape` and no Metal golden exists for it. The refusal is the correct behaviour at this boundary — the batched form is admitted by the schedule layer and declined by name at the layer that has no body for it — rather than a gap that silently produces a wrong kernel.

The remaining work, with the design derived on this branch so the next lane does not re-derive it:

1. **`crates/tiler-ir/src/kernel/lower.rs`.** `CooperativeContractionPlan` carries scalar `workgroups_m/_n`, `output_m/_n`; it needs a batch prefix. Note `workgroups_m` is **constructed and never read** today, so the field set is already not what the emitter uses. The workgroup ordinal is already linear (`wg = gid / threads`), so rank-N is a longer divide/modulo chain and needs no new KIR operation.
2. **Source-driven addressing, which is where the latent defect above is closed.** Emit each operand offset as the sum of `stride * coordinate` over its declared `ContractionAxisSource` list, with the operand's own row-major strides. At rank two this emits exactly today's `row * K + k` and `col * K + k` — the same operation multiset, so the measured `contract_tiled` kernel's cost does not change — and at rank four it handles the value structure's middle-contracted `[g, s, d]` right operand that the hardcoded form computes wrongly. Do **not** route this through `emit_offset` against the linear output index: that would add a divide and a modulo per operand per round to a kernel whose timing is a retained measurement.
3. **`crates/tiler-ir/src/kernel/verify.rs`.** `BlockedGeom`, `IndexRole`, `classify_binary`, `normalize_coord`, and `axis_guards` form an abstract interpreter over the emitted index arithmetic that proves the emitted guards *are* the blocked map. It is hardcoded rank two (`IndexRole::WgM/WgN/LocalM/LocalN/Row/Col`) and needs per-axis roles. **This is the highest-risk piece in the remainder** — a subtly wrong widening here admits a wrong kernel with no other check behind it — and it warrants the strongest model and an independent derivation.
4. **`crates/tiler-metal`.** No change beyond a new rank-four golden and its `GOLDENS` registration: the emitter is generic over KIR operations, has zero rank-two hardcoding, and the Metal dispatch is 1-D by construction (`MTLSize::new(grid_threads, 1, 1)`), so grid rank is not a constraint at any output rank.

**A defect worth its own ticket regardless of whether the batched lowering proceeds:** the discarded access maps in `emit_cooperative_contraction`. A region whose cooperative-contraction operand sources are not the canonical `[M, K]` / `[N, K]` orientation verifies today and lowers to a kernel addressing it by a different relation. `verify_blocked_operand_roles` on this branch closes the *block-axis* half of that hole; the operand-layout half stays open until the addressing is source-driven.

## Scopes

No scope was added. The branch touches `crates/tiler-ir` (`implementation/ir`) and `tickets/` (`project/tickets`, shared) and nothing else. `implementation/metal` was declared and **not used** — the delivery needs no Metal change, and the golden that would need one is in the remainder above.
