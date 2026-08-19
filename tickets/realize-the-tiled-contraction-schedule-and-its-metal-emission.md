---
id: realize-the-tiled-contraction-schedule-and-its-metal-emission
title: Realize the tiled contraction schedule and its Metal emission
status: deferred
priority: p1
dependencies: [admit-a-cooperative-tile-over-shared-operands, admit-guarded-output-tails-for-cooperative-contraction, admit-a-two-dimensional-cooperative-staging-relation, reclassify-language-model-work-as-a-conformance-track]
related: [realize-the-strict-contraction-on-metal, realize-the-contraction-through-the-appendable-direct-path, integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/ir, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model, deferred, class-generic-capability]
---
## User-visible outcome

The `tiled` realization the [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) selects compiles through the ordinary entry point as a retained alternative beside `direct`, refuses `K` not a multiple of its tile width with a typed reason that has been watched firing, and emits an MSL body carrying no fused multiply-add on its accumulation path.

## What remains once its cooperative dependencies land

This is the *fourth* wall [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) derived plus the work that was always this ticket's, and it is deliberately the cheapest of the four so it is not attempted first with a half-built vocabulary.

- **The topology dispatch.** `verify_intrinsic` (`crates/tiler-ir/src/schedule/builder/intrinsic.rs`, anchor `pub(super) fn verify_intrinsic`) dispatches on the scalar program: `StrictTensorContraction` reaches `verify_contraction` (`crates/tiler-ir/src/schedule/builder/contraction.rs`), which requires `ReductionTopology::Contraction` by `let … else`, and `verify_cooperative_semantics` (`crates/tiler-ir/src/schedule/builder/reduction.rs`) is reachable only from the four single-read reduction programs. A contraction therefore cannot carry a cooperative topology at all. A new `ReductionTopology` variant at appended tag `0x36` with its own semantic-verification arm is what admits it; the tag is an append that moves no earlier region's bytes.
- **The guarded output tail used by four retained rows.** The exact-divisible first pass deliberately refuses a partial output block. The retained kernel instead keeps the entire workgroup convergent, guards operand loads, and predicates the owning store at `M = 1` and `M = 10`; [`admit-guarded-output-tails-for-cooperative-contraction`](admit-guarded-output-tails-for-cooperative-contraction.md) must state and verify that relation before this ticket can claim all six correctness cells.
- **The schedule and the alternative.** `single_workgroup_tree_region` (`crates/tiler-compiler/src/physical.rs`) is the precedent — a constructor returning a typed `…Unavailable` decline the frontier records as a declined strategy rather than as an absence — and the tiled alternative follows it, offered beside `direct` rather than replacing it.
- **The `K ≡ 0 (mod 16)` precondition as a typed refusal, watched firing, never a pad.** `+0.0` is the strict sum's empty result and is not its bitwise-neutral padding; a K-padding schedule would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires. Refuse rather than acquire that obligation.
- **The two-allocation lowering.** `cooperative_plan`'s `let ([staging], [produce, consume]) = …` (`crates/tiler-ir/src/kernel/lower.rs`) admits one allocation and one visibility edge. Two allocations are already admissible at the schedule layer — `verify_cooperative_tile` loops over `tile.staging`, and `SynchronizationPoint::discharges`/`discharges_anti` do not read an edge's `staging` field, so one phase boundary discharges both edges and one round boundary both anti-dependencies — so this is an emission widening with no identity consequence.
- **The Metal emission of a multi-round two-allocation contraction body.** KIR already has multi-round *reduction* emission (`emit_loop_carried_cooperative` for a single staging allocation and contributor-split rounds). Landed Metal goldens for cooperative work are single-round (`cooperative_workgroup_reduction.metal`). The two-allocation contraction multi-round body — round loop, staged tiles, and barriers in MSL — is still this ticket's, with `spikes/scheduling/metal_contraction_vertical/kernels.metal` (`contract_tiled`) as the reference text and the existing golden idiom for the evidence.
- **No fused multiply-add on the accumulation path.** The flag is not sufficient — the L3 spike measured `simdgroup_multiply_accumulate` fusing under `-ffp-contract=off`, reproducing [finding 16](../docs/research/apple-targets/numerical-behaviour.md) — so the per-statement emission rule is what holds the line, and `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` is the evidence idiom.
- **Bit-comparison at all six L3 profile cells** against the retained `result_sha256`, with the staged reference oracle (`StagedStrictTensorContractionF32`) as the drift check. State the measurement boundary: a host comparison is not a dispatched one, and [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns the device.

**Correction — 2026-08-19 (paths only; every cited symbol re-located and every substantive claim retained).** The topology-dispatch bullet cited `crates/tiler-ir/src/schedule/builder.rs`, which no longer exists — the schedule-builder split replaced it with the `builder/` directory. The three verifiers it names live in three different submodules now and are cited individually above: `verify_intrinsic` in `builder/intrinsic.rs`, `verify_contraction` in `builder/contraction.rs`, `verify_cooperative_semantics` in `builder/reduction.rs`. `verify_cooperative_tile`, named in the two-allocation-lowering bullet without a path, is in `builder/tile.rs`. The dispatch relation, the appended-tag argument, and the identity consequence are unchanged.

**Do not "repair" `StrictTensorContraction` here.** ADR 0112 retired the *semantic operation key* `tiler::strict-tensor-contraction-f32@1` in favour of `tiler::tensor-contraction-f32@1`, but the *schedule* variant `ScalarProgram::StrictTensorContraction` is a different vocabulary and still exists under that spelling at this base (`crates/tiler-ir/src/schedule/builder/contraction.rs`, anchor `let ScalarProgram::StrictTensorContraction`). Renaming it in this ticket to match the retired key would substitute a new false claim for a true one.

## Numerical legality, already settled

The L3 `tiled` schedule preserves each thread's ascending left fold over its output's contributors; the K-chunk loop changes only the **memory schedule** (threadgroup tiles and barriers), not the reduction tree. L3 therefore attributes it uniquely to `strict_fold+ftz` and records it as **consuming no numerical permission**, the same attribution and byte-identical results as `direct`. Both remain strict-admissible. Reproduce: L3 legality row "`tiled` | **Yes**, consuming no permission"; kernels.metal "changes the memory schedule and nothing about the reduction". Do not require reassociation, and do not gate this schedule on `NumericalContract::FLUSH_AND_REASSOCIATE_F32` — that would falsely refuse it under strict/FTZ contracts where L3 proved it legal. (`a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` is a parallel-sum portfolio fixture under a reassociating contract, not a warrant for this schedule.)

**Correction — 2026-08-10.** Prior wording claimed the tiled schedule consumes **reassociation** by regrouping contributors into per-round chunks and is therefore admissible under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, citing `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` as numerical precedent. That is false for L3 `contract_tiled`: products still enter a single left-fold accumulator in ascending contributor order; the existing `CooperativeWorkgroup` *reduction* topology is what consumes reassociation (shared-output parallel sum), the inverse relation. Withdrawn. Live guidance is the paragraph above.

## Non-goals

The split alternatives, the matrix-instruction route, any opaque call, and any cost model. [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids substituting a differently-attributed realization to make a number better, and the L3 record states the measured price of not doing it.

## Activation triggers

Deferred behind the exact-divisible relation, its guarded-output-tail extension, and the already-landed two-dimensional staging relation. It becomes work only when both cooperative-contraction tickets are `done`, because the retained six-cell population contains exact and partial output blocks and this ticket may not silently substitute the direct realization for either.

## Closes when

A contraction of the profile's projection structure compiles through the ordinary entry point to a tiled Metal kernel, its results are bit-identical to the reference at every profile cell, the `K` precondition refuses with a typed reason that was watched firing, and the emitted module carries no fused multiply-add on the contraction's accumulation path.

## Trigger check log

- 2026-08-04 — **not fired.** The activation trigger is [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md) reaching `done`; it is still `deferred`, and this sweep found its own trigger only half fired — its dependency landed but the two public boundaries it needs are Tom's and unaccepted. Recheck: that ticket's status and its trigger-check log.
- 2026-08-09 — **not fired.** The two-dimensional staging dependency is `done`, but `admit-a-cooperative-tile-over-shared-operands` is now correctly `awaiting-decision`: the second cooperative relation and its ownership-proof kind remain Tom's public-boundary decision. This implementation stays deferred behind that answer rather than appearing runnable merely because the lower dependency landed.
