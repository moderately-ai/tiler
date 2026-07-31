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

## Review packet (2026-07-31)

**The two blocking facts are resolved.** `ProposalBody::KernelSubprogram` carries a typed `KernelSubprogram` of ordered `SubprogramStage`s instead of the `ReservedProposalSeam`, and `AdmittedImplementation` gained an `ImplementationBody::Subprogram` arm. `selection::reconcile_boundaries` needed **no** change: a subprogram's internal handoff never reaches a cover edge, so `derive_subprogram_boundary_contract` yields the same one-intermediate-in, one-output-out contract the serial reduction offers, and the "at most one intermediate per region" rule is satisfied rather than relaxed. `DeterministicBudgets::governed` widened `regions` 2 → 3 and `buffers` 3 → 4, and `verify_program`'s hardcoded minimums moved with them (2 → 3, 3 → 4) so the widening is enforced rather than decorative.

**A provider now reports what it withheld.** `PhysicalImplementationProvider::propose` returns a `ProviderOffer` (proposals plus `DeclinedStrategy` entries) instead of a bare `Vec`, and `FrontierRejection::StrategyDeclined` carries the typed `StrategyDeclineCause`. Without this channel the enumeration cannot distinguish "this provider does not implement splitting" from "this request's extents admit no split" — both retain exactly the serial alternative — and the ticket's "the split's absence explainable" is unsatisfiable. The refusal is decided from the contract *before* any region is built, because a region carrying `permits_reassociation: false` is rejected by the schedule verifier as `NumericalOrAccessRefinement`, which the frontier classifies as `FrontierError::MalformedProposal` and which fails the whole enumeration closed — reporting a caller's numerical choice as a Tiler defect.

**Identity effects.** The budget widening moves the canonical request subject, and therefore every artifact identity and cache entry derived from it, for **every** governed compilation — not only ones that assemble a split, because a budget is a property of the request rather than of the plan chosen for it. One pinned value moved: `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately` digest, `83b9baadbea45e19` → `09d719dd4c2c2f37`, recomputed on this tree. No other golden encodes these bytes; every other request-subject assertion in the corpus is relational. `PROPOSAL_IDENTITY_TAG` did **not** step: a subprogram's identity subject is a length-framed ordered fold of its stages' canonical region identities under the existing tag and the existing `PhysicalProposalKind::KernelSubprogram` discriminant, so no previously encodable proposal's bytes move.

**Measurement boundary — the split is enumerable and assemblable, and not yet reachable end to end.** A split consumes reassociation; the only registered contract permitting it (`governed_relaxed`) also permits contraction, and for the recognized serial-sum program — whose members mix multiply and add — `derive_fusion_legality` reports `unrealized-contraction` for *every* multi-member candidate, so no legal cover survives and `compile` returns `no-complete-plan`. That is pre-existing and pinned by `fusion_legality::tests::a_relaxed_mixed_arithmetic_region_still_needs_contraction_evidence`; it is a contraction-evidence question, not a reduction-splitting one. The compile path is therefore exercised for the *decline* (the strict-contract fixture records one `frontier.strategy-decline.v1`), and the admission, assembly, and oracle conformance are exercised against `enumerate_frontier` and `build_split_kernel_program` directly. Filed as `admit-a-reassociating-contract-without-contraction`.

**Out of scope, observed.** `docs/architecture.md:195` still says the bounded frontier "admits only checked `ScheduledKernel` values and rejects the other variants"; that was already false for `OpaqueCall` and is now also false for `KernelSubprogram`. `docs/architecture.md` is `contracts/foundation`, not this ticket's `contracts/optimizer`.

**Public boundary changes for Tom, none self-accepted.** No `tiler-ir` item changed and no `tiler-compiler` *public* item changed — `session`, `target`, `legality`, and `capability` are untouched. Every changed item is `pub(crate)` within `tiler-compiler`; the consequential crate-internal boundary is `PhysicalImplementationProvider::propose`'s new return type and the `ProposalBody::KernelSubprogram` payload.

## Graph maintenance

Keep artifact encoding and replay in `realize-parallel-reduction-strategies-on-metal` and preference in `calibrate-and-activate-parallel-reduction-selection`. If `implement-parallel-reduction-strategies` still reads as owning this enumeration, narrow it against this ticket rather than duplicating the outcome.

- `admit-a-reassociating-contract-without-contraction` owns the reachability gap above; until it lands, no compilation selects a split, which is also why `calibrate-and-activate-parallel-reduction-selection` cannot measure one yet.
- A ragged final partition remains unimplemented and refuses with a typed reason; if evidence arrives that prime or sub-four contributor extents matter on real workloads, file it with the loop-vocabulary extension it implies rather than widening `ContributorPartition::covers`.
