---
id: decide-the-canonical-staged-pass-access-spelling-for-coincident-rms-operands
title: Decide the canonical staged-pass access spelling for coincident RMS operands
status: todo
priority: p1
dependencies: [admit-the-rms-normalization-family]
related: [repair-retired-declared-input-order-authority-in-request-and-physical-comments]
scopes: [implementation/compiler, implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, identity, normalization, schedule]
---
## User-visible outcome

Tiler either retains the typed refusal for `rms_norm(x, x)` or accepts one exact canonical staged-pass access spelling for two semantic RMS operands which bind the same declared input. No implementation may admit the population by accidentally choosing between two local accesses and one coalesced access.

## Discovery — 2026-08-17, exact main `e8141d7decbb8204e7930421d0b1acedef9b4dd5`

**Fact — the semantic occurrence is legal and recognized.** RMS normalization has two operand positions, value and weight. The retained normalized subject can carry the same private `DeclaredInputOrdinal` in both positions.

**Fact — current physical construction refuses it explicitly.** `physical.rs`, anchor `value_input == weight_input`, returns no proposal. Its adjacent explanation that intrinsic schedule read-ordering would reject two spellings is false: `TensorRole::Input` is fieldless, and the intrinsic verifier admits multiple exact input access positions.

**Fact — checked association does not settle canonical local spelling.** `VerifiedScheduledRegion::declared_input_at(AccessOrdinal)` can project two positions to the same declaration. A two-access spelling preserves semantic operand positions and the governed RMS lowering's separate value/weight tensors. A one-access spelling shares one local read/leaf. Both can denote the same interface tensor, but they have different schedule/kernel structure and canonical identity.

**Inference — removing the guard without this decision would invent identity authority.** The choice affects request-to-region construction, governed index-refinement binding, schedule verification, kernel/program assembly, aliasing expectations, and identity. Pre-production compatibility does not choose which spelling is correct.

## Required decision packet

- Re-audit the exact accepted RMS semantic/law surface and every request, physical, schedule, kernel, assembly, interpreter/runtime, and identity consumer at the implementation base.
- Apply the Pareto gate to at least: status-quo typed refusal; two exact operand-position accesses both projected to one declaration; and one coalesced access with both semantic operand uses bound to it. Eliminate any spelling which loses operand association, silently changes aliasing, or cannot be verified and explained.
- For each survivor fix the exact public/private representation, validation and refusal ownership, canonical identity consequences, unsupported populations, host-memory effect, strongest counterargument, reversal evidence, and independent subject perturbations.
- Decide whether admission requires a domain/schema step or only new values in existing injective grammars; do not infer this from the absence of a new enum variant.
- Ask Tom one exact question only after independent review proves the frontier complete. Until then `value_input == weight_input` remains a deliberate fail-closed construction boundary.

## Non-goals

No general tensor-alias analysis, in-place mutation, operand deduplication policy, runtime buffer ownership redesign, or other staged-family widening is implicit in this decision.
