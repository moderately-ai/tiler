---
id: record-typed-refusals-for-uncovered-contraction-realizations
title: Record typed refusals for uncovered contraction realizations
status: todo
priority: p2
dependencies: []
related: [realize-the-attention-contractions-on-metal, admit-reassociated-contraction-schedule-alternatives, qualify-the-simdgroup-matrix-contraction-realization]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, explainability, contraction]
---
## User-visible outcome

A caller who asks why a contraction was realized by the direct fold and not by a split, a matrix instruction, or an opaque provider gets a typed decline naming *which* explanation applies — reassociation, permutation, or absent distributivity — rather than an absence.

## Why this exists

Filed 2026-08-22 by `worker-attention` as the enumerated remainder of [`realize-the-attention-contractions-on-metal`](realize-the-attention-contractions-on-metal.md), whose Required delivery asks for *"a refusal for every realization whose reduction topology is unstated or uncovered, naming reassociation, permutation, or the absent distributivity separately, because those are three different explanations."*

**Fact — the contraction arm records no decline at all today.** `govern_spelling`'s contraction case in `crates/tiler-compiler/src/frontier.rs` offers `contraction_region` and adds no parallel strategy, with the comment that splitting would consume the reassociation this family declares forbidden. That reasoning is correct and is exactly what should be *reported* rather than left implicit. Re-derive at your base.

**Fact — the vocabulary to say it already exists and needs no widening.** `StrategyDeclineCause` on the public `#[non_exhaustive]` enum already carries `NumericalPermissionRefused { dimension }` and `AlgebraicCapabilityUnsupported { dimension }`, which is the three-way split the ticket asks for without a new variant. Verify before adding anything public.

**Fact — the four uncovered realizations and their distinct grounds**, from the [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md): `ksplit_contiguous` needs reassociation; `ksplit_strided` needs reassociation *and* permutation, and is the measured demonstration that the two are different plans and not one; `simdgroup` delivers a fused multiply-add where ADR 0015's contraction permission is Forbidden *and* seeds its accumulator at `+0.0` where the profile declares no seed; `opaque_mps` is refuted against all twenty-two named topologies and cannot state its accumulation order at all.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict.
- Record one decline per uncovered realization, each naming its own ground. **Do not collapse them**: a caller told only "numerical permission refused" cannot tell a split from a matrix instruction.
- Prefer the existing decline vocabulary. If a new variant is genuinely required, that is a public-surface change and needs its own justification.
- One negative control: under a contract that *does* grant reassociation, the contiguous split's decline must change or disappear, so the decline is a function of the contract rather than a constant.
- Perturb each decline separately, subject not assertion, with quoted failure text.

## Non-goals

Offering any of these realizations — `admit-reassociated-contraction-schedule-alternatives` and `qualify-the-simdgroup-matrix-contraction-realization` own those; and the tiled alternative's own decline, which belongs to its offer ticket.

## Closes when

Each uncovered contraction realization records a typed decline naming its own ground, the three explanations stay separately named, the contract-sensitivity control holds, and the workspace gate is green.
