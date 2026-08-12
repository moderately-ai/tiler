---
id: separate-vector-operand-alignment-from-target-realization
title: Separate implementation access alignment from target realization identity
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-vector-lane-bindings-into-the-schedule-vocabulary, declare-cpu-vector-realization-facts-in-the-target-profile, define-plural-operation-specific-vector-realization-requirements, prove-the-first-real-fixed-vector-cpu-execution-approach]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/runtime, implementation/cpu, contracts/foundation, contracts/optimizer, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, alignment, applicability, runtime, public-boundary, decision, needs-tom]
---
## User-visible outcome

An implementation requiring alignment `A` accepts an actual bound address proved at any stronger compatible alignment and refuses an insufficient or unknown address before routing commits. The target profile does not multiply exact realization rows merely because two implementations of the same operation place different alignment demands on their operands.

## Source-first Fact audit — exact base `f199b26376612e4b39c35569b084dda4c67490ce`

1. **Verified.** `crates/tiler-compiler/src/boundary.rs`, anchor `A byte alignment of a boundary value's first element`, already has a checked power-of-two `ByteAlignment`; `alignment_subsumption_is_divisibility_in_one_direction_only` proves that 16 satisfies 4 and 4 does not satisfy 16. The mathematical relation is implemented correctly but is crate-private and stops at compiler boundary composition.
2. **Verified.** `frontier::bounded_requirements` and `bounded_guarantees` derive only the storage carrier's natural alignment. `derive_subprogram_boundary_contract` derives the complete boundary from the verified scheduled body, and `ImplementationProposal` explicitly has no way to state a selected implementation's stronger access requirement. Two physical implementations with the same verified body therefore cannot differ in alignment applicability today.
3. **Verified.** `KernelProgramBuilder::push_physical_value` checks a materialized value's declared alignment against its allocation, but `push_view` and `check_stage_accesses` do not derive the alignment guaranteed after a nonzero byte offset. A view at offset `O` can guarantee only `gcd(base_alignment, O)` when `O != 0`; carrying the base value's alignment unchanged overstates the addressed pointer.
4. **Verified.** Artifact `BindingRef::alignment`, anchor `Returns the byte alignment the bound storage must satisfy`, already encodes the correct runtime subject. `ArtifactProgramBuilder::check_bindings` currently fills it from `value.alignment()` rather than from the selected access requirement and does not account for the view offset. The wire field exists; its producer is incomplete.
5. **Verified.** `RuntimeAdapter::plan_dispatch`, anchor `The last chance to refuse`, receives every routed binding and may compare caller-supplied storage before commit, but it returns `()` and the loader never receives alignment evidence. `allocate_dispatch` runs after the one-way commit, so adapter-owned storage can only be preflighted by an allocator guarantee and then asserted after allocation as a terminal defect if the allocator breaks that guarantee.
6. **Verified.** The pinned AArch64 `stdarch` implementation of `vld1q_f32` and `vst1q_f32` uses unaligned reads/writes and has offset-slice tests. The first real NEON form therefore requires the F32 carrier's natural four-byte alignment, not an invented 16-byte requirement. A real 16-byte-address guarantee must still satisfy that four-byte requirement; no fake aligned-only backend is needed to exercise the relation.
7. **False in the old packet.** Comparing one abstract operand proof before selection is not the complete boundary. Alignment must survive selected-implementation identity, program views, artifact binding construction, actual adapter storage planning, and post-commit allocator assertions. The previous three-bullet ticket was under-scoped and could have reported success while runtime still ignored the artifact's alignment obligation.

## Recommended authority split

Keep five orthogonal subjects and join them explicitly:

| Subject | Owner | Meaning |
| --- | --- | --- |
| Natural alignment floor | storage carrier vocabulary | The minimum alignment any well-formed access of that carrier needs. |
| Access alignment requirement | selected physical implementation | The minimum alignment this exact implementation requires for each exact stage/buffer access; it may strengthen but never weaken the natural floor. |
| Alignment guarantee | program allocation and view | What the actual value/view is statically guaranteed to provide after its byte offset. |
| Binding alignment requirement | artifact entry binding | What the final bound address for this exact entry slot must satisfy. |
| Binding alignment evidence | runtime storage plan | What the already-existing external address provides, what a future allocator contract guarantees, or `Unknown`. |

The first three use one checked, opaque power-of-two `ByteAlignment` vocabulary, with role-specific `AlignmentRequirement` and `AlignmentGuarantee` wrappers so callers cannot reverse the comparison. `AlignmentGuarantee::satisfies(requirement)` owns divisibility. `AlignmentGuarantee::after_offset(offset)` owns the checked effective-view derivation; offset zero preserves the guarantee and any nonzero offset reduces it to the largest power of two dividing both base guarantee and offset. No rank, tensor extent, provider key, target row, or pointer is stored in this scalar.

A physical proposal carries a complete ordered access-alignment population keyed by its exact scheduled stage and kernel buffer position. The host validates cardinality, ownership, natural floors, and canonical order, then strengthens the derived boundary requirements and the allocations/guarantees needed by writes. The provider cannot weaken a floor, add a nonexistent slot, claim a guarantee without an allocation satisfying it, or omit a slot. The exact population enters `ImplementationProposalIdentity`; it is not a cost estimate, target fact, or optional default.

The selected-plan projection exposes only the derived per-entry/per-slot requirements needed by the neutral build layer. It does not expose backend instructions, private proposal bodies, costs, rejected alternatives, or caller-constructible selected evidence. The artifact builder checks each requirement against the addressed program view's effective guarantee and writes the requirement into the existing binding-alignment field. No new manifest field or artifact grammar is needed.

`RuntimeAdapter::plan_dispatch` returns a complete bounded alignment report for the route it just planned. Each exact entry/slot reports `BindingAlignmentEvidence::ObservedAddress(guarantee)`, `AllocatorGuaranteed(guarantee)`, or `Unknown`. The loader owns the requirement-versus-guarantee comparison, rejects missing/duplicate/foreign rows as malformed adapter output, and treats `Unknown` or an insufficient guarantee as a typed pre-commit applicability refusal. For allocator-owned storage, `allocate_dispatch` must assert that the returned address meets the preflight guarantee; a violation is a post-commit adapter/allocator defect and never fallback.

The real CPU route observes external addresses from its actual host storage and reports allocator guarantees from its actual allocator policy. The first NEON image states four-byte requirements for contiguous F32 accesses, proves an exactly four-byte-aligned address and a stronger 16-byte-aligned address both execute identically, and proves insufficient/unknown cases stop before the CPU executor. No simulator, mock provider, fake device, Candle path, or arbitrary aligned-only instruction counts as evidence.

## Why this dominates the alternatives

1. **Recommended split above.** It is complete across selection, identity, views, artifact, and runtime; fails before commit; preserves target-row MECE; and adds only bounded linear work over already bounded stages and bindings.
2. **Put alignment inside `VectorRealizationSubject`.** Rejected: it multiplies exact target declarations for an implementation-side applicability difference and equality makes a 16-byte guarantee fail a four-byte row rather than satisfy it.
3. **Derive natural alignment only from the carrier.** Correct for the first NEON form but incomplete for a future selected implementation with a stronger real requirement; it cannot distinguish two provider variants over the same operation and body.
4. **Put the requirement in the target-neutral schedule or KIR alone.** Rejected as the sole owner: two backend/provider realizations may consume the same schedule/KIR while choosing memory forms with different alignment requirements. The selected physical implementation, not semantic coverage, owns that distinction.
5. **Check alignment only in a runtime adapter.** Rejected: planning, dominance, artifact identity, and explanation would all omit a hard applicability condition, and each adapter could implement a different comparison.
6. **Treat unknown as natural alignment or zero.** Rejected: either silently admits an unproved address or creates a numeric sentinel with no alignment meaning.

## Strongest counterpoint and reversal evidence

A provider can understate what its emitted instruction actually requires; the host can prove the natural floor and structural correspondence but cannot infer a native instruction's hidden contract. This is not repaired by moving the same claim into a target row. The accepted provider is trusted native code, and the real protection is an exact provider/versioned execution subject, CPU image grammar, translator-to-payload cross-check, runtime address proof, disassembly, and independent conformance. Reverse to a host-derived-only requirement if every admitted backend representation proves from its decoded operation vocabulary that no implementation choice can strengthen alignment; the first extensible provider seam does not establish that invariant.

## Identity and compatibility

`ByteAlignment` retains its fixed-width canonical spelling. Moving the checked vocabulary and deriving effective view guarantees need not change existing bytes. Adding the complete selected access population changes `tiler.compiler.physical-implementation-proposal.v2` to the next domain because internal as well as boundary accesses become identity-bearing. Existing natural-only plans may keep their schedule, KIR, program, and artifact binding bytes; their selected proposal and every subject that folds it move deliberately. The artifact manifest needs no schema step because its existing binding field already encodes the requirement. Any implementation that changes an old valid artifact byte for another reason must name and version that reason separately.

## First implementation tranche

1. [`admit-typed-byte-alignment-and-effective-program-view-guarantees`](admit-typed-byte-alignment-and-effective-program-view-guarantees.md) owns the shared checked vocabulary, role-safe relation, view-offset derivation, and natural-access program verification.
2. [`carry-complete-access-alignment-requirements-on-physical-proposals`](carry-complete-access-alignment-requirements-on-physical-proposals.md) owns the complete selected implementation population, proposal identity, boundary strengthening, selected projection, and compiler program-allocation consequences.
3. [`derive-artifact-binding-alignment-from-selected-access-requirements`](derive-artifact-binding-alignment-from-selected-access-requirements.md) owns the neutral build join, effective-view check, existing artifact field, and no-schema proof.
4. [`prove-planned-binding-alignment-before-routing-commit`](prove-planned-binding-alignment-before-routing-commit.md) owns the complete runtime report, loader comparison, adapter/allocator contract, and typed pre/post-commit failures.
5. [`prove-the-first-real-fixed-vector-cpu-execution-approach`](prove-the-first-real-fixed-vector-cpu-execution-approach.md) consumes all four through real NEON payload bytes and actual host storage before the public vector KIR returns for acceptance.

No item is `deferred`. The decision node is parked only for Tom's public-boundary acceptance; every child is filed with exact dependencies and becomes ordinary implementation work when its prerequisites are done.

## Required negative controls

- Perturb zero, non-power-of-two, equal, stronger, weaker, and unknown alignments independently.
- Offset an aligned base by zero, one natural element, and one nonconforming byte count; verify the derived guarantee rather than an assertion.
- Omit, duplicate, reorder, weaken, and assign a requirement to the wrong stage/buffer; each reaches a distinct typed failure.
- Change only the selected alignment requirement and prove proposal/artifact identity movement without changing the target realization row.
- Return an incomplete runtime report, an insufficient observed address, an insufficient allocator guarantee, and a post-commit allocation that breaks its reported guarantee; prove the first three are fallback-permitted and the last is terminal.
- Execute the same real NEON image from an exactly natural address and a stronger address; inspect real vector instructions and compare both outputs independently with `tiler-reference`.

## Decision request

Accept the complete authority split and first tranche above; revise a named carrier; or keep vector execution unavailable. Accepting this ticket authorizes the exact public boundaries and graph, not their implementation or a broader vector form.

## Closes when

Tom accepts the exact authority split and public/runtime carrier, the four tranche tickets are released, and every vector/CPU dependent points at the implementation evidence it actually consumes rather than at this decision record alone.
