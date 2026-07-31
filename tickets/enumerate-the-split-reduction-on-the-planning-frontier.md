---
id: enumerate-the-split-reduction-on-the-planning-frontier
title: Enumerate the split reduction as a retained frontier alternative
status: in-progress
priority: p1
dependencies: [implement-the-target-neutral-multi-pass-reduction-strategy]
related: [calibrate-and-activate-parallel-reduction-selection, realize-parallel-reduction-strategies-on-metal, implement-parallel-reduction-strategies]
scopes: [implementation/compiler, implementation/ir, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, scheduling, reductions, frontier]
claimed_from: todo
assignee: loop-frontier
lease_expires_at: 1785524875
---
## User-visible outcome

A compilation whose numerical contract permits reassociation retains the multi-pass split as an enumerated frontier alternative beside the serial reduction, with distinct identities, explain records for both the accepted and the rejected candidate, and a program assembler that wires the two passes plus their `PartialReduction` contract into one verified kernel program. Nothing here makes the split *win* — `calibrate-and-activate-parallel-reduction-selection` owns preference.

## Why this is its own ticket

`implement-the-target-neutral-multi-pass-reduction-strategy` landed the target-neutral halves — `ReductionTopology::MultiPass` region verification, the program-scope `PartialReduction` contract under `tiler.kernel-program.v6`, the exact reference oracles, and the compiler's region constructors (`governed_partition`, `partial_reduction_region`, `final_reduction_region`) with the request-subject binding. Frontier enumeration was deliberately not absorbed because it is blocked on two facts found during that work:

- **Fact:** a split realizes one semantic occurrence with two dispatches. The bounded profile expresses that only through the reserved `ProposalBody::KernelSubprogram`, and `selection::reconcile_boundaries` admits at most one intermediate per region — both must be extended before a two-dispatch proposal can be enumerated at all.
- **Fact:** `DeterministicBudgets::governed` fixes `regions: 2` and `buffers: 3`, and `verify_request` requires both exactly. A three-stage split program (pointwise, partial, final) needs three regions and four buffers. Widening a governed budget is a deliberate decision with its own identity consequences, not a test-enabling edit — the prior worker removed a compiler-side program assembler rather than widening it in passing.

## Implementation keys

Extend the proposal vocabulary and reconciliation to carry a two-dispatch alternative; widen the governed deterministic budgets deliberately, with the widening recorded and its identity effects stated; reintroduce the program assembler that emits producer/partial/final stages, the data dependency, and the `PartialReduction` declaration; and produce explain records for the split as accepted and as rejected (infeasible partition, forbidden reassociation). Compare assembled-program outputs against `tiler_reference::strict_partitioned_sum` — the oracle that answers for the split's chosen order — never against the serial fold under relaxed comparison.

A ragged final partition stays out of scope: `ContributorPartition::covers` requires an exact product because a ragged tail needs a second constant trip count the structured-kernel loop vocabulary does not carry. If evidence arrives that ragged splits matter (prime or sub-four contributor extents on real workloads), file that as its own capability with the loop-vocabulary extension it implies.

## Closes when

The frontier enumerates serial and split alternatives with distinct identities for a reassociation-permitting request; the governed budgets admit the three-stage program by an explicit, recorded widening; explain output covers accepted and rejected split candidates with typed reasons; assembled programs verify and match `strict_partitioned_sum`; every new check is perturbation-proved; and targeted tests plus the batch gate pass. Metal realization and calibrated selection remain in their own tickets.

## Graph maintenance

Keep artifact encoding and replay in `realize-parallel-reduction-strategies-on-metal` and preference in `calibrate-and-activate-parallel-reduction-selection`. If `implement-parallel-reduction-strategies` still reads as owning this enumeration, narrow it against this ticket rather than duplicating the outcome.
