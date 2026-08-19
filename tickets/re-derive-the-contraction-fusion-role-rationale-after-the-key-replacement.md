---
id: re-derive-the-contraction-fusion-role-rationale-after-the-key-replacement
title: Re-derive the contraction fusion-role rationale after the key replacement
status: todo
priority: p2
dependencies: []
related: [repair-the-research-records-the-key-replacement-and-splits-falsified, replace-the-serial-sum-contributor-fields-with-the-exhaustive-source]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, documentation, numerics]
---
## User-visible outcome

The comment block that justifies why a contraction receives `PrologueCarryingOrderedReduction` argues from facts the tree actually has, so a later reader deriving a neighbouring role is not reasoning from a deleted constant.

## Why this exists

Filed 2026-08-19 from the post-chain multi-lens audit and verified first-hand by the coordinator at `de18ebdb`.

**Fact — the recorded rationale rests on a declared fact ADR 0112 deleted.** `crates/tiler-compiler/src/fusion_legality.rs`, anchor `` `reassociation-permitted: false` withholds ``. The block rejects two alternative roles on that ground: `ExtremumShiftedOrderedReduction` "would claim a freedom … this family's own `reassociation-permitted: false` withholds", and `ElementwiseArithmetic` would "derive `Legal` under a reassociating contract that grants exactly the regrouping this family forbids". Neither constant exists — `grep -rn "CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED\|CONTRACTION_F32_FACT_PERMUTATION_PERMITTED" crates/` returns nothing at this base — and `reduction_descriptor_record` (`crates/tiler-ir/src/semantic/contraction.rs`) now declares that row `"permission-gated"`. The family does not forbid regrouping; it gates it on the ceiling.

**Fact — behaviour does not follow the stale premise, so this is a prose defect at a high-scrutiny site, not a bug.** The discharge consults the **contract**, never the family's declared fact: `fusion_legality.rs` reads `if !has_reduction || matches!(contract.reassociation, NumericalPermission::Forbidden)`, so a reassociating contract still lands on `Unknown` / `"unproven-reassociation"` fail-closed. The coordinator verified this independently before filing. **Do not change behaviour to match the comment, and do not change the comment to describe behaviour you have not read.**

**Why it is worth a re-derivation rather than a word swap.** The role assignment is still right, but the *reason* it is right has changed: it was "the family forbids regrouping", and it is now "the discharge is contract-driven and fails closed on an ungranted permission, and the family's own row is permission-gated rather than a standing refusal". A one-word edit that substitutes `permission-gated` into the old sentence would leave an argument that no longer follows from its premise, which is the shape AGENTS.md warns produces a *different* false claim in place of the original.

## Coordination

Exclusive `implementation/compiler`. At filing time that scope is held by `replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`; this ticket queues behind whatever lane holds it. That carrier rewrites recognition and subject encoding and may move or rewrite the surrounding block, so **re-audit the anchor at your actual base before editing** — if the block has already been rewritten, report what it now says instead of restoring the version this ticket describes.

## Required work

- Re-audit both Facts at your actual base and report a per-Fact verdict before editing.
- Rewrite the rationale so each rejected alternative is rejected on a premise the tree has, naming the contract-driven discharge and the permission-gated row explicitly. Preserve the conclusion (the role assignment) and the two rejections; replace only their grounds.
- Read the neighbouring role derivations in the same file for the same stale premise before concluding — one instance of a pattern obliges checking its siblings. Report what you found and what you found clean.
- Confirm by reading that the discharge path is unchanged by your edit, and state the check that would show it: the comment is not load-bearing, so no test should move. If any test moves, stop and report.

## Non-goals

Any behavioural change, any change to `NumericalPermission` handling or the fail-closed discharge, re-litigating ADR 0112, and repairs outside `crates/tiler-compiler/`. The documents and tickets carrying the same stale constants are three sibling tickets' scopes.

## Closes when

The rationale argues from facts the tree has, the sibling scan is reported, no test moves, and the touched-package `cargo check`, Clippy-with-warnings-denied, and rustdoc gates are green.
