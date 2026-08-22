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
