---
id: admit-reassociated-contraction-schedule-alternatives
title: Admit reassociated and permuted contraction schedule alternatives
status: blocked
priority: p2
dependencies: [implement-parallel-reduction-strategies, realize-the-contraction-through-the-appendable-direct-path, decide-the-fixed-strided-contributor-membership-vocabulary]
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

Hard dependency on [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) re-pointed to [`realize-the-contraction-through-the-appendable-direct-path`](realize-the-contraction-through-the-appendable-direct-path.md). Tiled Non-goals exclude the split alternatives; tiled parks on the cooperative free-index tile public boundary, which this K-split outcome does not name. The foundation is the landed direct contraction path plus the parallel-reduction portfolio patterns; free-index tiled emission is a sibling alternative, not a prerequisite. The shared private `split_family` table still returns `None` for `StrictTensorContraction` — that remains this ticket's work. Tiled stays `related` for shared contraction-topology lessons only.

**Source-first correction — 2026-08-11, exact base `b30e384497682c91771fcf93c5ce6854054d39a3`.** The graph repair originally named `multi_pass_family` / `cooperative_family`; [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md) was followed by the private consolidation at `eb0b7514`, so the current source has one `split_family` table instead. Its `ScalarProgram::StrictTensorContraction { .. } => None` arm preserves the substantive exclusion. Reproduced with `rg -n 'multi_pass_family|cooperative_family|fn split_family|StrictTensorContraction' crates/tiler-ir/src/schedule/builder.rs`.

## Source-first stop — 2026-08-11

**Fact — the retained contiguous and strided kernels differ in contributor membership, not in staged-partial arrival.** The complete eight-case producer defines `topology_contiguous_split` by slicing `products[lane * span : (lane + 1) * span]` and `topology_strided_split` by slicing `products[lane :: split]`; both then fold the partials in ascending lane order. The retained observation for `split_topology` is `bb1d0683` for `ksplit_contiguous` and `bb1d0672` for `ksplit_strided`, and the corpus attribution retains only `contiguous_split+ftz` and `strided_split+ftz` respectively. Reproduced with `rg -n 'topology_contiguous_split|topology_strided_split|split_topology' spikes/scheduling/metal_contraction_vertical/contraction_probe.py spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/semantics-{observations,attribution}.tsv`.

**Fact — every currently admitted split states contiguous membership.** `ContributorPartition` says partition `p` covers `p * contributors_per_partition .. (p + 1) * contributors_per_partition`; `ReductionTopology::MultiPass` inherits that contract, and `ReductionTopology::CooperativeWorkgroup` says participant `p` folds that contiguous range before staged partials are combined in ascending participant order. `verify_cooperative_semantics` proves the same coverage, `emit_partition_contributor` emits the same offset, and `push_schedule` encodes only the partition counts plus the cooperative arrival. Reproduced with `rg -n 'contiguous contributor range|emit_partition_contributor|fn push_schedule|ContributorPartition' crates/tiler-ir/src/schedule/{model.rs,builder.rs} crates/tiler-ir/src/kernel/lower.rs`.

**Fact — the existing permutation-bearing arrival values do not state the retained strided tree.** `ContributorArrival::AscendingParticipant` is the retained merge order but is documented as consuming no permutation because it composes contiguous partitions. `NondeterministicArrival` and `AtomicAccumulation` describe unfixed or atomic arrival, not fixed lane-strided membership; `verify_cooperative_semantics` refuses both under `CooperativeTileRule::UnadmittedArrival` even after permutation is granted. Reproduced with `rg -n 'enum ContributorArrival|requires_permutation|UnadmittedArrival|arrival != ContributorArrival::AscendingParticipant' crates/tiler-ir/src/schedule/{cooperative.rs,builder.rs,error.rs}`.

**Inference — the two-plan outcome is unstatable without a public and identity-bearing physical vocabulary decision.** Encoding a private strided implementation as `Contraction` would misstate a serial fold; encoding it as `MultiPass` or `CooperativeWorkgroup` would contradict their contiguous-membership contract; and choosing strided versus contiguous only inside private lowering would give two different trees one schedule identity. Extending `ContractionAxisSource::Contracted` to carry lane/stride arithmetic would instead widen the public access grammar and still leave the physical topology unnamed. Each route crosses [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md)'s requirement that the selected topology participate in physical-plan and artifact identity.

**No-edit outcome.** The worker stopped before implementation code as required. A contiguous-only change would not satisfy this ticket: it would omit the independently permutation-gated plan and its distinct missing-dimension refusal. [`decide-the-fixed-strided-contributor-membership-vocabulary`](decide-the-fixed-strided-contributor-membership-vocabulary.md) now owns Tom's exact public-carrier and append-only identity decision, and is a hard dependency rather than a related note.

## Closes when

Both alternatives exist, each is admitted only under its own permission with the refusal watched firing for the other, and the contiguous plan reproduces the spike's retained `contiguous_split` candidate bits on the eight-case corpus.
