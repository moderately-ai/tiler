---
id: derive-the-capability-set-for-search-discovered-flash-class-attention-kernels
title: Derive the capability set for search-discovered flash-class attention kernels
status: todo
priority: p2
dependencies: []
related: [decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode, calibrate-and-activate-parallel-reduction-selection, accept-adr-0100-multi-round-reduction-composition]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A research record that answers, capability by capability, what Tiler must grow before a caller who states a naive Llama-style attention program — `QKᵀ`, softmax, `×V`, written as registered semantic operations with no flash/sage/streaming vocabulary anywhere in the IR — can have the optimizer *discover* an implementation in the FlashAttention class, with the numerical delta stated as contract rather than taken silently. The record is a map from each required capability to an existing seam, an existing ticket, or a newly filed one — not an implementation, and not a promise of parity.

## Why this exists, and why now

**Fact — the architecture already decomposes the target.** FlashAttention is not one algorithm; it is (1) a materialization decision (never build the `S = QKᵀ` intermediate), (2) a streaming two-level reduction schedule, and (3) an algebraic rewrite of the softmax normalization (the shifted-max rescaling identity) that changes the reduction structure. Component 1 is the cover/materialization search the compiler already enumerates. Component 2's vocabulary largely landed this week: cooperative tiles, loop-carried staging, the two-dimensional staging relation ([ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md), implemented), and the multi-round two-level composition ([ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md), accepted 2026-08-05). Component 3 has no home yet, and it is the crux.

**Fact — the compiler searches declared rewrites; it does not invent algebra.** [ADR 0099](../docs/decisions/0099-project-an-elementary-familys-per-point-body-from-one-shared-statement.md) made single-statement projection the standing rule, and the optimizer contract admits logical alternatives "only when the effective permissions authorize the regrouping". [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) shows the discipline refusing a freedom nothing needed. So "inventing" flash attention decomposes honestly into: operations *declare* local algebraic identities once (softmax's normalization commutes with rescaling by the exponential of a shifted max; the fold admits an online form under stated permissions), and the global algorithm emerges from search over fusion, materialization, schedule, and those declared rewrites. The record must say exactly what the declaration vocabulary is, who owns it, and what proves a declared identity sound.

**Fact — the cost layer is the largest absence.** The 2026-08-05 research-status audit recorded it against source: no cost model, cost estimate, or ranking type exists anywhere in `crates/`; selection is a legality join. Search cannot prefer the flash-shaped candidate over the naive one without a cost authority, and the roadmap's bootstrap-cost-model record is where that thread currently ends. Reproduce with the audit's own grep in [the scheduled-region model's implementation-status section](../docs/research/scheduling/scheduled-region-model.md).

**Fact — the numerical boundary is the differentiator, not an obstacle.** Every SOTA attention kernel changes bits relative to the naive spelling. Under this repository's contracts that change is only reachable when the caller grants it, which converts "our kernel is fast" into "our kernel is fast *and* states what numerical freedom bought it". The softmax family already separates its max-fold and sum-fold legality facts; the record must derive what additional permission vocabulary an online-softmax rewrite consumes, and whether SageAttention-class quantized attention is expressible as a quantization contract with accuracy obligations on the existing `require_elementary_accuracy` shape.

## The derivation the record owes, by axis

- **Rewrite and permission vocabulary.** What typed, operation-owned declaration lets softmax (or the exponential) state the shifted-max rescaling identity and the streaming-fold form? What numerical permission does firing it consume, and how does the permission compose with [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md)'s declined distributivity and the order-contract dimensions? What evidence class proves a declared identity sound (`SoundProof`, exhaustive finite, empirical) — and what refuses an unsound declaration?
- **Fusion and materialization search.** Does the current cover enumeration already contain the no-`S` candidate for the three-operation attention chain once the fusion roles exist, or does the epilogue wall (`admit-elementwise-epilogues-over-a-materialized-intermediate`) and the fusion-role gap (`admit-a-fusion-role-for-the-tensor-contraction`) bound it first? Map the exact tickets.
- **Schedule composition.** Which of the flash schedule's remaining constructs are missing after ADR 0100 — the tile-blocked write map and bijectivity proof (`admit-a-two-dimensional-cooperative-staging-relation`'s successors), the tiled contraction realization, symbolic extents for the growing context axis — and which existing deferrals carry them?
- **Cost authority and calibration.** What is the smallest cost model that ranks naive-versus-streaming credibly on Apple Silicon: analytic (bytes moved, occupancy), calibrated (measured per-construct costs, the M3/M4 measurement discipline), or learned? What does `calibrate-and-activate-parallel-reduction-selection` already own, and what would a flash-shaped decision additionally need? Where does simulation sit — is a memory-traffic simulator worth its maintenance against direct measurement on the two host rows we own?
- **Search strategy and budget.** At what candidate-space size does exhaustive enumeration stop and guided search start, and what does the deterministic-budget vocabulary already bound? What does explain owe so a rejected flash-shaped candidate is a readable ledger entry rather than silence?
- **Conformance oracle.** What does the reference layer owe so a rewritten (bit-different) implementation is checkable — an enclosure-based oracle for the permitted delta, per the certified-arithmetic machinery in `tiler-reference::accuracy`, or per-contract golden regeneration? The decoder-layer assembly's zero-differing-elements evidence is the naive baseline; what is the oracle for the *permitted-divergence* case?
- **Information the system lacks.** Target facts not yet in any profile (shared-memory bandwidth rows, simdgroup matrix capabilities, occupancy limits), and which ledger owns each.

## The expected outcome class, stated so nobody overdelivers

This is a graph-augmentation ticket, and that is its success condition, not its consolation prize. The likely deliverable is a record whose every axis ends in *filed, refined, or re-edged tickets* — new research questions, corrected dependencies on existing ones, deferrals with honest triggers — and possibly zero code, zero ADRs, and zero experiments run. A worker who maps all seven axes to well-edged tickets and defers every question that lacks evidence has completed this ticket; a worker who forces a premature ADR or experiment to satisfy the research-outcome discipline has misread it — a deferred question with a reconsideration trigger *is* one of the four sanctioned outcomes, and for a capability derivation this far ahead of the implementation frontier it is the expected one on most axes. Where an axis genuinely resolves to something stronger (an existing seam already suffices, or a bounded experiment is cheap and decisive), take it; nothing here forbids substance — the point is that substance is not owed.

## Explicit non-goals

Implementing any of it; adding any IR vocabulary; an attention-specific anything in the compiler (the record must show the general capability and treat attention as the worked example); performance claims (nothing here measures a kernel); reopening accepted numerical decisions — the record works within ADR 0095's decline and derives what *additional* permissions a caller would grant, not a relaxation of the defaults.

## Closes when

The record exists under `docs/research/program-planning/` with contract-conforming frontmatter and catalog rows, every required capability is mapped to an existing seam, an existing ticket, or a ticket filed by this work with correct edges (deferrals carrying trigger logs), the worked example walks the naive attention chain through each axis showing where today's tree refuses and what closes each refusal, the four maturity claims are kept apart per capability, and the record ends in the AGENTS.md research outcomes — each axis reaching a contract update, an accepted decision, a bounded experiment, or an explicitly deferred question with a reconsideration trigger, never an open-ended note.
