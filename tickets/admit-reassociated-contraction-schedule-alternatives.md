---
id: admit-reassociated-contraction-schedule-alternatives
title: Admit reassociated and permuted contraction schedule alternatives
status: in-progress
priority: p2
dependencies: [implement-parallel-reduction-strategies, realize-the-contraction-through-the-appendable-direct-path]
related: [reduction-semantics-contract, implement-analytical-component-cost-model, realize-the-tiled-contraction-schedule-and-its-metal-emission, enumerate-the-split-reduction-on-the-planning-frontier, admit-a-reassociating-contract-without-contraction]
scopes: [implementation/compiler, implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, reductions, contraction, numerics]
claimed_from: todo
assignee: sol-contraction-schedules
lease_expires_at: 1786439940
---
## User-visible outcome

A caller whose contract permits reassociation gets a contracted-axis-split contraction schedule, and a caller whose contract permits reassociation but *not* permutation gets the contiguous split and not the strided one — two different plans gated on two different permissions, rather than one "parallel reduction" that quietly consumes both.

## The distinction, measured

**Measurement — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md).** Two split kernels were implemented and measured. `ksplit_contiguous` partitions the contracted axis into contiguous intervals merged in ascending order and is attributed uniquely to `contiguous_split+ftz`; `ksplit_strided` partitions it into strided subsets and is attributed uniquely to `strided_split+ftz`. They return different bits: at the spike's `split_topology` case, `0xbb1d0683` against `0xbb1d0672`. [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md) fixes why — reassociation without permutation may combine only contiguous contributor intervals in order, and a lane-strided partition reorders leaves — and this is the measurement that makes it a plan difference rather than a definitional one.

**Measurement — and one of them is the fastest thing measured at decode.** At the complete vocabulary projection (`M=1, N=151936, K=1024`) `ksplit_contiguous` ran in 4,247 microseconds on the M3 Pro bench host, ahead of `MPSMatrixMultiplication` at 4,418 and the strict `direct` kernel at 4,757. The cell is bandwidth-bound at about 146 GB/s and those candidates are all near the same bound, so the win is small — but it is the shape where the split is worth having, and prefill is the shape where it is not: the same kernel is roughly 5x to 7x *slower* than the tiled strict kernel at `M >= 128`.

## Required delivery

- Two schedule alternatives, each declaring the exact permission it requires, with an infeasibility that names the missing dimension — reassociation or permutation — rather than reporting a generic illegality. A rejection that says "reassociation forbidden" for a strided plan is the wrong explanation.
- The split precondition `K` a positive multiple of the split width as a typed refusal.
- Partial state per [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md): the seed attaches once at the root-facing boundary and never per lane, and a partial carries `has_value` unless nonemptiness or proven-neutral padding is established.
- **This implementation is not the strategy.** The spike's split kernels dedicate 32 lanes to one output and idle most of them at large `M`, which is a schedule written to isolate a reduction topology rather than to be fast. Their prefill numbers bound that implementation and are not evidence against split reductions; a split that also tiles the free indices is unmeasured.

## Non-goals

Distributivity, in either direction. Regrouping a contraction *chain* is a different dimension that no contract Tiler can express grants, and its rejection must name distributivity rather than reassociation.

## Graph repair — 2026-08-10

Hard dependency on [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) re-pointed to [`realize-the-contraction-through-the-appendable-direct-path`](realize-the-contraction-through-the-appendable-direct-path.md). Tiled Non-goals exclude the split alternatives; tiled parks on the cooperative free-index tile public boundary, which this K-split outcome does not name. The foundation is the landed direct contraction path plus the parallel-reduction portfolio patterns; free-index tiled emission is a sibling alternative, not a prerequisite. `multi_pass_family` / `cooperative_family` still return `None` for `StrictTensorContraction` — that remains this ticket's work. Tiled stays `related` for shared contraction-topology lessons only.

## Closes when

Both alternatives exist, each is admitted only under its own permission with the refusal watched firing for the other, and the contiguous plan reproduces the spike's retained `contiguous_split` candidate bits on the eight-case corpus.
