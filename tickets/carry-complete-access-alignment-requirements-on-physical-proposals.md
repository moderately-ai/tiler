---
id: carry-complete-access-alignment-requirements-on-physical-proposals
title: Carry complete access-alignment requirements on physical proposals
status: todo
priority: p1
dependencies: [separate-vector-operand-alignment-from-target-realization, admit-typed-byte-alignment-and-effective-program-view-guarantees, accept-the-installed-physical-provider-public-surface]
related: [publish-occurrence-bound-selected-physical-implementation-evidence, establish-vector-execution-form-numerical-authority]
scopes: [implementation/ir, implementation/compiler, implementation/build, contracts/optimizer, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [alignment, backend-providers, applicability, identity, public-boundary]
---
## Status repair — 2026-08-19, blocked with no surviving ground

Found by a board sweep for `blocked` tickets whose every declared dependency is `done`; seven turned up and this is one. **The body stated no blocking reason at all.**

**Verified by the coordinator at `4a813d21`:** every declared dependency — `separate-vector-operand-alignment-from-target-realization`, `admit-typed-byte-alignment-and-effective-program-view-guarantees`, and `accept-the-installed-physical-provider-public-surface` — is `status: done`, and each carries an explicit Tom-acceptance marker (`## Accepted decision` / `Tom accepted` / `RESOLVED — accepted`). So the ground this was parked on is gone, and the status was simply never flipped. Moved to `todo`; no other field changed.

**NOT verified, and it is the next worker's first task:** that this ticket's Facts are current. It predates several landed migrations — `tiler.kernel-program` v12→v13, `tiler.artifact-program` v20→v21, manifest (20,0)→(21,0), the retired contraction key, four module splits, and the decoded-input accessor becoming `static_shape() -> Option<Shape>`. Per the stale-Facts rule, re-audit every Fact at your own base and report a per-Fact verdict before editing; **repair the ticket and report the repair rather than working around a false Fact.**

## User-visible outcome

Two physical implementations of the same logical operation may state different real memory-alignment requirements without multiplying target facts, and the compiler selects either only when every exact access can satisfy it.

## Facts at filing base `f199b26376612e4b39c35569b084dda4c67490ce`

- **Verified.** `ImplementationProposal` currently carries body, target applicability, and cost only. The host derives one natural boundary alignment from the storage carrier, so a provider has no safe spelling for a stronger real access requirement.
- **Verified.** `TensorRole::Input` has an ordinal, but boundary roles alone are not a complete key: a subprogram has several stages and internal accesses that never appear at its external boundary. The requirement population must be keyed by exact stage and kernel buffer position.
- **Verified.** `ImplementationProposalIdentity` already folds provider, body, applicability, derived boundary, and feasibility under `tiler.compiler.physical-implementation-proposal.v2`; the complete access population is missing from that subject.
- **Verified.** The accepted installed-provider public surface exposes only scheduled-kernel proposals. Extending that constructor is a deliberate source break in a pre-production tree, not a compatibility overload with an implicit natural default.

## Required delivery

- Add an opaque, complete, canonical `AccessAlignmentRequirements` value with one requirement for every scheduled stage/buffer in execution order. Construction refuses missing, duplicate, foreign, misordered, zero, non-power-of-two, and below-natural entries.
- Require the value in `ImplementationProposal::scheduled_kernel`; no optional field or old overload remains. `BaselineImplementation` exposes the host-derived complete natural population so a specializing provider can copy it and strengthen exact slots without re-deriving buffer order.
- Re-verify the population after the proposed body re-enters host schedule/KIR verification. A requirement can only strengthen the host-derived natural floor. Read requirements strengthen the admitted boundary requirements; write requirements force the selected program allocation/guarantee to be at least as strong.
- Retain the complete population on `AdmittedImplementation`. Extend the compiler-minted selected-plan projection with exact per-entry/per-slot requirements for `tiler-build`; callers and providers cannot construct selected evidence.
- Encode the complete population in proposal identity under `tiler.compiler.physical-implementation-proposal.v3`. Update the owning domain ledger and every proposal/plan/portfolio/request/explain pin that folds it; do not step schedule, KIR, or target-profile domains.
- Explain an alignment miss with provider, proposal/variant, stage, buffer, tensor role, required, and guaranteed values. It is an applicability/boundary refusal, not target infeasibility or a cost loss.

## Strictness boundary

The host proves structure, cardinality, natural floors, and composition. It cannot infer a native instruction's hidden alignment rule. The CPU translator and image grammar must cross-check the selected requirement against the provider-versioned execution variant, and independent disassembly/conformance must falsify understatement. A provider that understates a backend instruction remains defective trusted native code; moving the same unprovable claim to a target row would not make it host-derived.

## Required evidence

- The governed scalar population states complete natural requirements and remains semantically unchanged.
- A real F32 vector proposal states four bytes for every contiguous NEON access; a 16-byte guarantee satisfies it and a two-byte guarantee does not.
- Strengthen exactly one read and one write requirement in separate fixtures; prove boundary composition, selected allocation alignment, proposal identity, and explain cause each move at that slot only.
- Omit, duplicate, reorder, weaken, and attach a requirement to a nonexistent stage/buffer; each produces a distinct typed malformed-proposal failure.
- Install two providers with identical verified bodies and different alignment populations; both remain distinct, neither borrows the other's applicability or numerical evidence, and canonical ordering is independent of installation order.

## Non-goals

Target realization declarations, actual runtime addresses, artifact encoding, cost ranking changes, new region shapes, opaque calls, or a fake aligned-only backend.

## Closes when

Every admitted implementation carries a complete verified access-alignment subject through identity, boundary composition, selected program allocation, public selected evidence, and explanation.
