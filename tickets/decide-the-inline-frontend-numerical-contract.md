---
id: decide-the-inline-frontend-numerical-contract
title: Decide the numerical contract every inline expansion compiles under
status: awaiting-decision
priority: p2
dependencies: []
related: [compose-the-numerical-contract-from-its-decided-dimensions, package-a-multi-entry-bundle-from-one-expansion, prototype-inline-aot-integration-proof, reach-a-reassociation-permitting-contract-from-a-bound-metal-declaration, calibrate-and-activate-parallel-reduction-selection]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [frontend, numerics, public-boundary, decision]
---
## User-visible outcome

`tensor!` compiles every region under a numerical contract Tom chose, rather than under the one that happened to be the only admissible name.

## Why this is now a decision, and was not before

**Fact — the activation trigger fired on its own terms.** `crates/tiler-macros/src/aot.rs` states `CONTRACT` because the region grammar carries no numerical statement, and `only_one_numerical_contract_is_admissible_for_the_bound_declaration` existed to say when that stopped being a derivation: "if the declaration ever admits a second, this test fails and the frontend has a real choice to put to Tom instead of a silent one it already made." It failed under `compose-the-numerical-contract-from-its-decided-dimensions`, and its successor `the_bound_declaration_admits_the_two_flushing_contracts` pins the new pair.

**Fact — the pair.** Against `BoundMetalCompileDeclaration::first_macos_apple9`, the admissible named contracts are now `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` and `NumericalContract::FLUSH_AND_REASSOCIATE_F32`. The strict and permit-reassociation contracts are still refused on `InputSubnormals` — this hardware's `f32` arithmetic flushes in every measured math mode — and the relaxed one is still declined by fusion legality for a multiply adjacent to an add. Nothing about the profile changed; the contract vocabulary did.

**Inference — the two are different meanings, not two settings.** The composed contract widens exactly one further dimension: ordered regrouping of one same-operation operand sequence. Under it a reduction may be split or folded as a workgroup tree, so its result may differ from the flush-only reading in the last bits; under the flush-only contract it may not. Both are legal statements about what a program computes, and neither is stricter in every respect than the other.

## What each option enables and prevents

- **Keep `FLUSH_SUBNORMALS_TO_ZERO_F32`** (today's behaviour). Every expanded program keeps the exact meaning it has been delivering, so no artifact's *semantics* move. It also permanently forecloses every parallel reduction strategy for inline regions: the split and the single-workgroup tree both consume regrouping, so `tensor!` will never select one however large the reduction gets.
- **Move to `FLUSH_AND_REASSOCIATE_F32`.** Inline regions become able to select a parallel reduction on the measured Apple row, which is what `calibrate-and-activate-parallel-reduction-selection` needs a consumer for. The cost is that every expanded program's stated meaning changes — a caller who wrote `tensor!` yesterday gets a contract permitting regrouping today, without writing anything different.
- **Let the region grammar state it.** The correct long-run shape and out of scope here: it needs a grammar surface, a spelling, and its own boundary acceptance. Both options above remain the default for a region that states nothing.

**Recommendation.** Keep `FLUSH_SUBNORMALS_TO_ZERO_F32` until the grammar can state a contract. Silently widening what every existing expansion means is the one direction a numerical default must not move on its own, and the parallel-reduction consumer can be reached by a region that states the wider contract explicitly once the grammar admits one.

## Closes when

Tom states which contract `tiler_macros::aot::CONTRACT` names, the module documentation records the decision with its date and venue, and `the_bound_declaration_admits_the_two_flushing_contracts` asserts against the chosen one.
