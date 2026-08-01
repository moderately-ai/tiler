---
id: admit-the-first-typed-synchronization-point-and-atomic-target-authority
title: Admit the first typed synchronization point and atomic target authority
status: in-progress
priority: p1
dependencies: [replace-or-justify-the-barrier-count-axis, represent-cooperative-workgroup-reduction-dataflow]
related: [construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/foundation, contracts/optimizer, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [synchronization, feasibility, target-profiles, correctness]
claimed_from: todo
assignee: worker-sync-authority
lease_expires_at: 1785589250
---
## User-visible outcome

Tiler can represent and verify its first synchronized schedule without treating the number of barrier operations as a target capacity. A synchronized implementation is executable only when one provenance-bearing target fact establishes the exact synchronization realization it consumes; a schedule with no synchronization remains vacuously admitted without such a fact.

## Facts and boundary

**Fact:** `replace-or-justify-the-barrier-count-axis` removes the numeric barrier capability and makes every current KIR barrier intrinsically unauthorized because the implemented schedule owns no synchronization point, phase, placement, participant set, visibility contract, or convergence proof.

**Fact:** the reserved `BarrierSpec` fields for execution scope, memory scope, fenced spaces, and ordering are not a schedule obligation and cannot establish convergence or target support. Equal field shapes do not make those concepts one authority.

**Inference:** independently declared target facts for scope, fence, visibility, or ordering would permit false composition: each component could be supported in some realization while their conjunction is unsupported. The target fact must therefore be atomic over the complete synchronization subject.

**Proposal:** prove the target-neutral contract on the meaningful cooperative workgroup reduction dataflow delivered by `represent-cooperative-workgroup-reduction-dataflow`, with one bounded synchronized schedule and a synthetic governed target authority. A barrier inserted into the current pointwise/global-linear program is eliminated as closing evidence because it is semantically redundant or divergent under predication. Do not promote Metal support from source acceptance or from a backend spelling; `realize-parallel-reduction-strategies-on-metal` owns primary backend evidence.

## Implementation keys

- Add a schedule-owned stable synchronization-point identity and bind it to an explicit phase and placement in the normalized schedule.
- Define the operation kind rather than assuming every synchronization point is a control barrier. Keep asynchronous copies, split-phase barriers, collectives, atomics, and inter-dispatch dependencies distinct until their own contracts are admitted.
- Define the complete participant set and execution scope, visibility obligation, fenced memory spaces, ordering, and convergence proof. State how each field is constructed and verified rather than accepting a caller assertion that a point is convergent.
- Make KIR synchronization reference the exact schedule point it realizes. Verify exact operation kind, participants and scope, phase and placement, visibility, fences, ordering, and convergence before deriving any target requirement.
- Introduce one atomic target synchronization-realization fact over that complete subject, including its availability phase and structurally attributed provenance. Do not let independently true component facts satisfy it, and do not infer it from a language version, successful compilation, backend spelling, or numeric operation count.
- Keep absence canonical: a schedule with no synchronization emits no synchronization requirement, target fact, explain row, or artifact field.
- Include every retained synchronization dimension and authority revision in schedule, KIR, target-profile, feasibility-rule, kernel, program, and artifact identity at the layer that owns it. Recompute domain and schema versions on the tree this work lands into; never copy a pinned value from an independently based branch.
- Preserve hard feasibility separately from cost. Unsupported or missing synchronization authority is `Unknown` or a typed rejection before executable-frontier admission; latency remains a cost fact and cannot establish legality.
- Carry the verified obligation and target realization through the artifact-facing program without creating a caller-declared ABI field or a second editable authority.

## Required evidence

- A zero-synchronization schedule succeeds against a sparse target profile containing no synchronization fact, and explain output contains no manufactured zero row.
- One bounded synchronized schedule reaches verified KIR and artifact construction only with an exact matching atomic target realization.
- Removing the target realization produces `Unknown` or the named fail-closed rejection before executable-frontier admission.
- One test per dimension proves that mismatched point identity, phase, placement, operation kind, participants, execution scope, visibility, fenced spaces, ordering, or convergence cannot satisfy the schedule.
- Identity mutation tests change each retained dimension and the target authority revision independently and observe every identity layer that owns that fact change.
- Canonical encode/decode and adversarial artifact tests prove the synchronization record cannot be reordered, partially omitted, or substituted while retaining identity.
- Every new check is perturbed once and observed failing. Run targeted per-package `cargo nextest run -p ...` and per-package Clippy while iterating, then `make full` once at the completed batch boundary.

## Closes when

The target-neutral vertical has one fully typed synchronized schedule whose KIR refinement, feasibility, explain, identity, and artifact paths agree; zero synchronization remains vacuous; every incomplete or mismatched authority fails closed with a typed cause; focused tests and `make full` pass; and Tom has reviewed the consequential public schedule, target-profile, and artifact boundaries.

## Graph maintenance

This ticket depends on `replace-or-justify-the-barrier-count-axis` for the fail-closed zero-synchronization baseline and on `represent-cooperative-workgroup-reduction-dataflow` for the first schedule that actually consumes synchronization. Its relation to `construct-and-bind-the-first-authoritative-metal-compile-profile` is non-blocking: that profile remains truthful without a synchronization row, and this ticket must not invent Metal evidence to broaden it. `implement-the-single-workgroup-synchronized-reduction-strategy` consumes the accepted point, and `realize-parallel-reduction-strategies-on-metal` owns backend qualification. Update `docs/ir.md`, `docs/compiler/fusion-and-scheduling.md`, `docs/artifact-abi.md`, the identity ledger, and any open-question entry whose status changes.
