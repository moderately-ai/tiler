---
id: state-the-search-constant-provenance-the-caps-audit-found-bare
title: State the search-constant provenance the caps audit found bare
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

**Amended 2026-08-08 — the `MetalHostPredicate::ALL` half is already done and is not this ticket's to do.** `size-the-four-hand-written-metal-all-arrays-from-their-types` landed it in `crates/tiler-metal/`, a crate this ticket does not scope (`implementation/compiler` only); the array is now declared `[Self; core::mem::variant_count::<Self>()]`. What remains here is the search-layer half alone. Original text follows.

Every production bound in the search layers carries either a derivation or a stated-bound-with-owner sentence, and `MetalHostPredicate::ALL` is derived from its variant count instead of a hand-written 7 beside the sibling that proves why hand-writing is the hazard.

## Why this exists (caps audit 2026-08-06 — headline verified twice: ZERO silent truncation in crates/; the residual is provenance)

The audit's counted table: 43 bare production constants and nine bare search-budget values beside five exhaustively-derived ones in the same struct, in a codebase whose own best examples (the budgets derivation idiom, cover.rs:1521's argued exclusion) set the bar. Plus: only one of three search layers publishes an always-on exhaustiveness fact. None of these is a behaviour defect — every bounded search reports a typed stop through explain — so this is one provenance pass, not N fixes.

## Closes when

The ALL constant derives from variant_count; each bare bound carries its classification sentence (derived / stated-with-owner); the exhaustiveness-fact asymmetry is either closed or stated at the two silent layers.
