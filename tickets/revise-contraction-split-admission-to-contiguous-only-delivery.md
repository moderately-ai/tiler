---
id: revise-contraction-split-admission-to-contiguous-only-delivery
title: Revise contraction split admission to contiguous-only delivery
status: todo
priority: p2
dependencies: [decide-the-algebraic-capability-authority-for-contraction-splits]
related: [admit-reassociated-contraction-schedule-alternatives, decide-the-semantic-order-contract-for-relaxed-contractions]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

[`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) is revised to match the accepted successor contract: contiguous membership is the only reachable delivery, the lane-strided alternative moves behind its future-generation trigger, and the required refusal vocabulary names the algebraic and numerical causes separately.

## Why this exists

The 2026-08-18 acceptance in [`decide-the-semantic-order-contract-for-relaxed-contractions`](decide-the-semantic-order-contract-for-relaxed-contractions.md) chose the reassociation-only successor and its downstream item (5) requires the admission ticket to be revised so contiguous membership is the only reachable delivery, but no ticket carried that revision. The reopened algebraic-authority packet additionally fixed the refusal contract the revision must inherit. The admission ticket's current `User-visible outcome`, `Required delivery`, and `Closes when` still demand both alternatives ("Both alternatives exist"), which is unsatisfiable under the accepted contract because the successor descriptor's permutation maximum is `unsupported`.

## Required revision

- Reduce the deliverable to the contiguous split; move lane-strided admission behind its trigger — an accepted fold-commutativity declaration in a future successor key generation plus independently resolved permutation permission — and record that trigger in the admission ticket, which does not yet state it (*corrected by independent review 2026-08-18: the trigger is recorded in the accepted semantic packet's downstream item (6) and the reopened authority packet, not in the admission ticket*). Keep the preserved attempt and the membership-vocabulary decision as evidence.
- Carry the reopened authority packet's verifier contract: descriptor decode plus effective-profile resolution as the two-fact join; a new appended algebraic `StrategyDeclineCause` variant with a stable reason key naming the missing dimension, distinct from `NumericalPermissionRefused`; lane-strided refused algebraically, not numerically; provider-output recheck before frontier admission.
- Update `Closes when` so it no longer requires the lane-strided plan, while retaining the contiguous-plan bit-reproduction obligation on the eight-case corpus and the witness/explanation membership-projection repair from the 2026-08-17 review stop.

## Obligations inherited from the replacement migration's independent review — 2026-08-19

The ADR 0112 landing (`e61fbc60`, merged) shipped reserved vocabulary this revision's implementation graph must make live and tested, per review findings 4 and 5 at that commit; name these in the revised admission ticket so they are not rediscovered:

- **`StrategyDeclineCause::AlgebraicCapabilityUnsupported` has no construction site at the merged base.** Its 0x06 encoding, `algebraic-capability-unsupported` reason, and `CapabilityResolution` explain routing are landed but unreachable. The split-admission implementation is what constructs it (lane-strided membership refused algebraically); its perturbations — including proof that the algebraic and numerical sources report distinctly and never collapse — belong to that implementation, with failure text quoted.
- **The witness's regular-split branch is implemented but unreachable and untested** (`partitioned_chain_nodes`, `MalformedPartition`, `AmbiguousRealization`, and the split-combiner staging refusals in `crates/tiler-ir/src/program/contraction_witness.rs`). A construction bug there would surface only as a *different legal tree* — exactly the class the witness exists to pin — so split-path witness tests, including each named refusal, must land before any split plan consumes that code.
- Minor, same file: the evaluator's witness-`K` revalidation reuses `Tree(RootCoverage)` off-label for a contributor-count disagreement; when the split path gains its tests, consider a typed mismatch variant rather than the borrowed spelling.

## Closes when

The admission ticket's outcome, delivery, refusal vocabulary, and closing conditions are consistent with the accepted reassociation-only contract and the accepted algebraic authority, with the lane-strided remainder explicitly parked behind its trigger rather than silently dropped, and with the three review-inherited obligations above carried into the revised ticket's own delivery requirements.
