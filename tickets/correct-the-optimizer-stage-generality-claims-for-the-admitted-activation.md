---
id: correct-the-optimizer-stage-generality-claims-for-the-admitted-activation
title: Correct the optimizer stage-generality claims for the admitted activation
status: review
priority: p2
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary]
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [docs, optimizer, correction]
claimed_from: todo
assignee: agent-opt-claims
lease_expires_at: 1785934519
---
## User-visible outcome

`docs/compiler/optimizer.md` stops telling a reader that `tiler::silu-f32@1` is refused at the request boundary, which it is not since 2026-08-04.

## The exact stale spans

**Fact — audited by reading the file at the commit that admitted the activation.** Two sections make claims the tree now refutes:

1. **"What each stage is general over today", the stage-1 paragraph.** It says the boundary "refuses a vocabulary, not a shape: an operation no region can spell, …". That sentence survives, but the enumeration of *which* operations no region can spell has moved: `tiler::silu-f32@1` is now projected into `PointwiseF32Node` from `crates/tiler-compiler/src/elementary.rs`'s single statement of its composition, and only `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` remain refused — because what they lack is a `LogicalAccess` spelling of their access relation rather than a per-point body.

2. **"Lowering capability resolution and index-region refinement", maturity boundary.** Its claim that the resolution/refinement pair is unconditional is unaffected. What is newly true and unstated is the *composition* the activation's admission established: because the boundary's projection and the governed index-access lowering are driven from one statement, refinement's proof that the emitted region realizes the occurrence is also evidence about the projection. That was observed rather than argued — perturbing the shared statement was watched failing at `compile.lowering.refinement-refused` before any region was scheduled — and it belongs in this contract because it is the reason a projecting boundary does not create a second unchecked authority.

There is a third addition the file does not currently have anywhere: **a program carrying a registered elementary family now places a hard accuracy obligation on its target.** `request::require_elementary_accuracy` assesses each distinct obligation per target, before numerical-contract resolution, and refuses with the refusing authority's stable key. That is feasibility rather than cost and belongs beside the four surfaces the optimizer may consult.

## Why this is a separate ticket

`ticketsplease.toml` maps `docs/compiler/**` to `contracts/optimizer`. The ticket that admitted the activation declares `implementation/compiler` exclusively, and `contracts/optimizer` was held by a live sibling at the time, so the correction could neither be made from that branch nor scheduled against a scope another worker held.

## Closes when

The stage-1 enumeration names the two families that still refuse and the reason each does, the refinement section states the composition the shared statement establishes, and the elementary accuracy obligation is stated as a feasibility question with its own refusal — each with the exact code location a reader can check it against.

## Outcome, 2026-08-05

All three corrections landed in `docs/compiler/optimizer.md`, each verified against the source at the base `561dfe0b` before it was written rather than transcribed from this ticket's claims.

1. **Stage 1, "What each stage is general over today".** A new paragraph after the stage-1 one states the refused set from `elementwise_family` (`crates/tiler-compiler/src/request.rs`), which classifies exactly `tiler::add-f32@1`, `tiler::multiply-f32@1`, and `tiler::silu-f32@1`; the activation's admission as a *projection* of its per-point body through `elementary::silu_point_body`; and `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` as what still refuses, with the reason read from `tiler_ir::schedule::LogicalAccess` (`crates/tiler-ir/src/schedule/model.rs`) — five variants, no reindex map, and `ScalarBroadcast` the only broadcast. Both structural families hold a registered index-access capability among the nine `governed_index_access_capabilities` returns, which is what makes the wall a region-vocabulary one.
2. **"Lowering capability resolution and index-region refinement", maturity boundary.** Two paragraphs: the composition — one statement in `elementary.rs`, two sink implementations (`GovernedElementarySink`, `PointwiseExpressionSink`), so `legality::refine_index_region`'s proof is evidence about the projection — and the measurement that established it, bounded to the one perturbation that produced it.
3. **"The four surfaces the optimizer may consult".** The elementary accuracy obligation is stated *beside* the four rather than as a fifth, because it is asked before any alternative exists: `request::require_elementary_accuracy` per target, before numerical-contract resolution and again in `readmit_candidate`, refusing with `RequestError::UnrealizedElementaryAccuracy` carrying `ElementaryAccuracyRefusal::diagnostic_code` — `accuracy.elementary.no-installed-realization` or `accuracy.elementary.unrefined-realization`. Framed as feasibility, with `assess_elementary_accuracy`'s one-directional conservatism as the reason it can never be traded against an estimate.

### Two claims this ticket carried that the tree refutes

**The stage-1 span was not stale; it was silent.** This ticket's item 1 and its parent's "`docs/compiler/optimizer.md` still lists all three families as refused" both describe an enumeration the file did not contain. At `561dfe0b` the stage-1 paragraph named no operation family at all — it says only "an operation no region can spell" — and `git show 561dfe0b:docs/compiler/optimizer.md | grep -n 'silu\|reindex\|broadcast'` returns five hits, none of them a family key and none in that section (they are the equivalence-group, normalization, and region-rule lists at lines 273, 274, 316, 317, and 367). So no sentence was corrected: the paragraph gained the enumeration it never had. Nothing in the file was telling a reader that `tiler::silu-f32@1` is refused, which makes the user-visible outcome as stated unreachable and the real defect an absence.

**`compile.lowering.refinement-refused` is not a string in the tree.** `grep -rn 'compile.lowering.refinement-refused' crates/` returns nothing. What exists is `LoweringError::Refine`'s stable reason `"refinement-refused"` (`crates/tiler-compiler/src/lowering.rs`) composed with the phase `"lowering"` by `lowering_failure` (`crates/tiler-compiler/src/pipeline/planning.rs`); `LoweringError`'s own `Display` renders `compile.lowering.refinement`, and the `RequestError` a caller sees renders `compile.unsupported.lowering.refinement-refused`. The doc therefore cites the typed value and its two construction sites rather than the composed spelling, which no site emits.
