---
id: state-the-search-constant-provenance-the-caps-audit-found-bare
title: State the search-constant provenance the caps audit found bare
status: closed
priority: p3
dependencies: []
related: [state-the-rule-that-a-deterministic-budget-is-a-derivation]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
closed_reason: superseded
closed_note: Its surviving six-cap provenance work and physical-plan policy question are consolidated into the broader deterministic-budget ticket.
---
## User-visible outcome

**Amended 2026-08-08 — the `MetalHostPredicate::ALL` half is already done and is not this ticket's to do.** `size-the-four-hand-written-metal-all-arrays-from-their-types` landed it in `crates/tiler-metal/`, a crate this ticket does not scope (`implementation/compiler` only); the array is now declared `[Self; core::mem::variant_count::<Self>()]`. What remains here is the search-layer half alone. Original text follows.

Every production bound in the search layers carries either a derivation or a stated-bound-with-owner sentence, and `MetalHostPredicate::ALL` is derived from its variant count instead of a hand-written 7 beside the sibling that proves why hand-writing is the hazard.

## Why this exists (caps audit 2026-08-06 — headline verified twice: ZERO silent truncation in crates/; the residual is provenance)

The audit's counted table: 43 bare production constants and ~~nine bare search-budget values beside five exhaustively-derived ones in the same struct~~ **— struck 2026-08-10: that 9/5 census was the original 2026-08-06 framing and is false at the tree; `DeterministicBudgets` partitions as eight derived / six literal (see Superseded and [`state-the-rule-that-a-deterministic-budget-is-a-derivation`](state-the-rule-that-a-deterministic-budget-is-a-derivation.md))** , in a codebase whose own best examples (the budgets derivation idiom, cover's argued exclusion and `is_exhaustive`) set the bar. Plus: only one of three search layers publishes an always-on exhaustiveness fact. None of these is a behaviour defect — every bounded search reports a typed stop through explain — so this is one provenance pass, not N fixes.

**Correction — 2026-08-10.** Why still carried the false 9/5 population after Superseded already stated eight derived / six literal. The 9/5 wording is historical caps-audit framing only; the corrected partition and live provenance work live on the superseding ticket. Dropped the `cover.rs:1521` line citation (line numbers rot) in favour of cover's argued exclusion / `is_exhaustive` surface.

## Closes when

~~The ALL constant derives from variant_count~~ **— struck 2026-08-08: `MetalHostPredicate::ALL` was landed by `size-the-four-hand-written-metal-all-arrays-from-their-types` in `crates/tiler-metal/`, a crate this ticket does not scope, and the metal-AOT counterparts landed the same day. This clause named completed work in the section governing closure; the outcome section was amended when the first half landed and this one was missed.** Each bare bound carries its classification sentence (derived / stated-with-owner); the exhaustiveness-fact asymmetry is either closed or stated at the two silent layers.

## Superseded — 2026-08-09

The surviving half is not an independent ticket. The complete source audit belongs to [`state-the-rule-that-a-deterministic-budget-is-a-derivation`](state-the-rule-that-a-deterministic-budget-is-a-derivation.md), which now owns the exact **eight derived / six literal** population, the omitted `region_candidates_per_seed`, the false governing-document classifications, and the `physical_plan_combinations` decision that can empty the portfolio. Keeping this node open would offer two workers the same compiler comments with contradictory 9/5 and 8/6 populations. Closed as superseded; no source work was performed here.
