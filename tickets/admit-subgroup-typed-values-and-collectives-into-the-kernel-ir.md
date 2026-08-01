---
id: admit-subgroup-typed-values-and-collectives-into-the-kernel-ir
title: Admit subgroup-typed values and shuffle collectives into the structured kernel IR
status: todo
priority: p2
dependencies: [accept-the-subgroup-execution-tier-adr, admit-subgroup-bindings-into-the-schedule-vocabulary]
related: [design-the-subgroup-execution-tier, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir, close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, ir, metal, subgroup, execution-hierarchy, public-boundary]
---
## User-visible outcome

The structured kernel IR can express a value that lives per subgroup lane and a shuffle that moves one, and the verifier discharges the obligations that creates — so a schedule stating a subgroup spread has a kernel-level construct to lower into rather than an emission gap.

## Why now

**Fact — the acceptance node releases nothing today.** [`accept-the-subgroup-execution-tier-adr`](accept-the-subgroup-execution-tier-adr.md):15 and `:31` both claim acceptance releases the implementation tickets gated behind it, and [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md):65 lists the four filed tickets, none of them an implementation ticket. This is the kernel-IR third of what makes that claim true.

**Fact — a shuffle needs no barrier, and that is the design's load-bearing negative result.** The 2026-08-01 addendum on [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md) records it from Metal Shading Language Specification 4.1 §6.10.2: "SIMD-group functions allow threads in a SIMD-group to share data **without using threadgroup memory or requiring any synchronization operations, such as a barrier**." A shuffle names its source lane and its destination register in one operation that is both the transfer and the ordering, so a shuffle-tree reduction derives no visibility edge, declares no synchronization point, and never reaches `barrier_call` at all. A design that routes a shuffle through a barrier would be wrong, not merely conservative.

**Inference — the reduction collectives are not the near-term construct, and the vocabulary must not pretend otherwise.** The subgroup tier record derives that subgroup *reduction* collectives are unusable for a separate reason — neither Metal nor WGSL states their combine order — so a collective admitted without a stated order would be a silently wrong result under an order-sensitive contract. The shuffle is admissible; the reduction collective is not, and refusing it explicitly is the correct outcome rather than a gap.

## Implementation keys

- Subgroup-typed values and the shuffle vocabulary the record derives, with the lane a shuffle reads named explicitly rather than inferred from position.
- The combine tree the record enumerates, whose order is *stated* in the IR because the hardware does not state it. An unordered reduction collective is refused by name.
- The lane identity's proof obligation lands as one concept with the CPU tier's, per [the subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md):391; read [`admit-lane-typed-values-and-masked-memory-into-the-kernel-ir`](admit-lane-typed-values-and-masked-memory-into-the-kernel-ir.md) against this ticket before choosing a shape.
- Identity encoding is additive at every site: appended tags only, no existing tag or field position moves, and the kernel identity domain does not step.
- If this widens `ExecutionScope` or `MemoryScope`, [`close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire`](close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire.md) is the check that must fire; land that tripwire first or update it in the same change.

## Required failure-path evidence

Each observed failing against an accepted neighbour: a shuffle whose source lane is outside the subgroup; a shuffle crossing a subgroup boundary; a combine tree whose order is unstated under an order-sensitive contract; a reduction collective relying on an unspecified hardware order; and a subgroup-typed value read from an invocation outside the subgroup that produced it.

## Non-goals

Schedule bindings (`admit-subgroup-bindings-into-the-schedule-vocabulary`, this ticket's dependency). Target profile declarations (`declare-metal-subgroup-realization-facts-in-the-target-profile`). MSL emission. The two-level subgroup-to-workgroup composition, which the ADR excludes and [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) owns — that is also the construct that fires `add-subgroup-memory-scope-when-collectives-land`, and this ticket must not fire it by accident. Any performance claim.

## Closes when

The constructs are admitted, every obligation above is checked by a check observed failing, the identity encoding is exhaustive, the record's worked examples are constructible with the verdicts it states, and every public shape has gone to Tom rather than been self-accepted.
