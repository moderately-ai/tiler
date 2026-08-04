---
id: realize-the-tiled-contraction-schedule-and-its-metal-emission
title: Realize the tiled contraction schedule and its Metal emission
status: deferred
priority: p1
dependencies: [admit-a-cooperative-tile-over-shared-operands, admit-a-two-dimensional-cooperative-staging-relation, reclassify-language-model-work-as-a-conformance-track]
related: [realize-the-strict-contraction-on-metal, realize-the-contraction-through-the-appendable-direct-path, integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/ir, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model, deferred, class-generic-capability]
---
## User-visible outcome

The `tiled` realization the [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) selects compiles through the ordinary entry point as a retained alternative beside `direct`, refuses `K` not a multiple of its tile width with a typed reason that has been watched firing, and emits an MSL body carrying no fused multiply-add on its accumulation path.

## What remains once its two dependencies land

This is the *fourth* wall [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) derived plus the work that was always this ticket's, and it is deliberately the cheapest of the four so it is not attempted first with a half-built vocabulary.

- **The topology dispatch.** `verify_intrinsic` (`crates/tiler-ir/src/schedule/builder.rs`) dispatches on the scalar program: `StrictTensorContraction` reaches `verify_contraction`, which requires `ReductionTopology::Contraction` by `let … else`, and `verify_cooperative_semantics` is reachable only from the four single-read reduction programs. A contraction therefore cannot carry a cooperative topology at all. A new `ReductionTopology` variant at appended tag `0x36` with its own semantic-verification arm is what admits it; the tag is an append that moves no earlier region's bytes.
- **The schedule and the alternative.** `single_workgroup_tree_region` (`crates/tiler-compiler/src/physical.rs`) is the precedent — a constructor returning a typed `…Unavailable` decline the frontier records as a declined strategy rather than as an absence — and the tiled alternative follows it, offered beside `direct` rather than replacing it.
- **The `K ≡ 0 (mod 16)` precondition as a typed refusal, watched firing, never a pad.** `+0.0` is the strict sum's empty result and is not its bitwise-neutral padding; a K-padding schedule would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires. Refuse rather than acquire that obligation.
- **The two-allocation lowering.** `cooperative_plan`'s `let ([staging], [produce, consume]) = …` (`crates/tiler-ir/src/kernel/lower.rs`) admits one allocation and one visibility edge. Two allocations are already admissible at the schedule layer — `verify_cooperative_tile` loops over `tile.staging`, and `SynchronizationPoint::discharges`/`discharges_anti` do not read an edge's `staging` field, so one phase boundary discharges both edges and one round boundary both anti-dependencies — so this is an emission widening with no identity consequence.
- **The Metal emission of a multi-round body.** None exists: the landed goldens are single-round. The round loop, the staged tiles, and the barriers in MSL are this ticket's, with `spikes/scheduling/metal_contraction_vertical/kernels.metal` as the reference text and the existing golden idiom for the evidence.
- **No fused multiply-add on the accumulation path.** The flag is not sufficient — the L3 spike measured `simdgroup_multiply_accumulate` fusing under `-ffp-contract=off`, reproducing [finding 16](../docs/research/apple-targets/numerical-behaviour.md) — so the per-statement emission rule is what holds the line, and `the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path` is the evidence idiom.
- **Bit-comparison at all six L3 profile cells** against the retained `result_sha256`, with the staged reference oracle (`StagedStrictTensorContractionF32`) as the drift check. State the measurement boundary: a host comparison is not a dispatched one, and [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns the device.

## Numerical legality, already settled

The tiled schedule consumes **reassociation** — it regroups the declared contributor sequence into per-round chunks without moving any contributor across the sequence — and no permutation. It is therefore admissible under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` against the bound declaration, and `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` is the precedent. `direct` stays the strict-admissible realization and is not retired by this ticket unless the measurement holds on the merged tree.

## Non-goals

The split alternatives, the matrix-instruction route, any opaque call, and any cost model. [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids substituting a differently-attributed realization to make a number better, and the L3 record states the measured price of not doing it.

## Activation triggers

Deferred behind both dependencies. It becomes work when [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md) is `done` — which itself requires [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) — because until then the tile's staged reads and its ownership are both unstatable and any schedule built here would be verified against a relation the vocabulary cannot express.

## Closes when

A contraction of the profile's projection structure compiles through the ordinary entry point to a tiled Metal kernel, its results are bit-identical to the reference at every profile cell, the `K` precondition refuses with a typed reason that was watched firing, and the emitted module carries no fused multiply-add on the contraction's accumulation path.
