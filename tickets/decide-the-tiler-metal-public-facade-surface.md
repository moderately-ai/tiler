---
id: decide-the-tiler-metal-public-facade-surface
title: Decide the tiler-metal public facade surface
status: in-progress
priority: p1
dependencies: [prototype-metal-kir-lowering, check-synchronization-realization-before-the-routing-commit, carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit]
related: [choose-one-owner-for-apple-target-vocabulary, realize-parallel-reduction-strategies-on-metal]
scopes: [implementation/metal, implementation/metal-aot, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, metal, facade]
claimed_from: todo
assignee: worker-metal-facade
lease_expires_at: 1786977676
---
## User-visible outcome

The `tiler-metal` crate has one accepted exact public facade, or one explicit typed deferral with a reconsideration trigger. Its crate-level and module-level maturity statements no longer leave a consequential public boundary orphaned behind terminal implementation tickets.

## Exact-current discovery — 2026-08-17 at `52de1babfe78f4bf3cac2c6e2bb8de50b1d401c5`

- **Verified — the whole crate is still deliberately draft.** `crates/tiler-metal/src/lib.rs`, anchor `Every public item in this crate is a reviewed *draft* boundary`, assigns the complete public facade to ADR 0074 section 7 review.
- **Verified — a newly consumed public subset repeats the hold.** `crates/tiler-metal/src/synchronization_requirement.rs`, anchor `exact surface returns to Tom`, says its exact public API is not accepted.
- **Verified — the only whole-facade proposal is terminal.** `tickets/prototype-metal-kir-lowering.md`, anchors `whole public surface of tiler-metal` and `remain open for Tom`, is `done`. The latter now leaves two unresolved questions: the portfolio form of `emit_translation_unit` and ownership of `MetalTargetFacts::buffer_binding_limit`. The Apple target-vocabulary duplication question was resolved separately.
- **Verified — no live owner exists.** `git grep -l -F 'whole public surface of \`tiler-metal\`' -- tickets` returns only the terminal prototype ticket. No current decision-queue row owns the facade.

This is not authority to delete the draft labels. Their statement is truthful; the missing piece is the live decision or explicit deferral they promise.

## Current-base correction — 2026-08-17 at `d002cd55406522922e5eb750c8c4d9033dde4469`

The discovery's blanket maturity verdicts are stale even though the quoted source text remains. `tiler_metal::applicability` contains exact subsets accepted by Tom across the macOS-host applicability decisions, while `direct_requirement` has an accepted surface whose final two visibility narrowings remain in `carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`. `synchronization_requirement` still has no exact-surface acceptance. Later contracts also settle buffer-capacity ownership and require multi-kernel translation-unit consumption semantically; only their exact Rust facade remains undecided. This ticket therefore depends on the visibility correction and must re-audit the complete current public census after it lands. It is not decision-ready from the four discovery bullets alone.

## Required decision packet

- Re-audit every public module, re-export, type, trait, function, constructor, accessor, error, and exhaustiveness promise at the exact packet base, including the direct-requirement, synchronization-requirement, applicability, target, emission, record, diagnostic, and target-correspondence surfaces and every cross-crate caller.
- Reconcile each surface with ADR 0074 section 7 and ADR 0075. Separate already accepted exact subsets from merely implemented drafts; a terminal implementation ticket is not acceptance provenance.
- Apply the Pareto-complete decision gate to the whole-facade survivors: accept the current facade, minimize it, split accepted narrow modules from a deferred remainder, keep a labelled draft with an explicit trigger, or any materially distinct current-source alternative. Eliminate cosmetic variants and any option that moves target, compiler, runtime, or device authority into this source-emission crate.
- Fix the exact included and excluded Rust surface, compatibility posture, error vocabulary, host-memory/runtime consequences, and downstream migration. State the strongest counterargument, reversal evidence, and independent subject perturbations for every survivor.
- Decide the two unresolved prototype questions from current evidence rather than inheriting their 2026-07 wording. Include every subsequently added public module so the result is not example-shaped around the original emitter.
- Update crate/module maturity prose, decision/navigation catalogs, and graph state only after Tom accepts an exact surface. If deferral survives, record it in `deferred` with a `## Trigger check log`; do not queue a non-ready packet.

## Stop boundary

This ticket is decision research only. It authorizes no public signature change, module move, compatibility shim, target-vocabulary consolidation, or production implementation before the exact packet passes independent review and Tom accepts it.

## Closes when

One exact current-source facade packet passes independent review and Tom accepts it, or a typed deferral records the evidence and trigger that makes future presentation actionable. Every live draft label then has a live owner or accepted disposition.
