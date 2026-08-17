---
id: admit-coincident-rms-operands-with-one-coalesced-pass-access
title: Admit coincident RMS operands with one coalesced pass access
status: blocked
priority: p1
dependencies: [decide-the-canonical-staged-pass-access-spelling-for-coincident-rms-operands]
related: [admit-a-scheduled-region-for-a-staged-elementary-family, accept-the-root-mean-square-scale-realization-law]
scopes: [implementation/ir, implementation/compiler, implementation/reference, implementation/build, implementation/artifact, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, normalization, schedule, identity, aliasing]
---
## Outcome

If Tom accepts the prerequisite decision, `rms_norm(x, x)` compiles through the same governed RMS law and provider as distinct-operand RMS while retaining two ordered semantic operand uses and one canonical coalesced dense pass access. The reference, refinement receipt, schedule, kernel, program, artifact, and runtime association remain exact and fail closed outside the accepted alias slice.

## Required implementation

- Widen only the RMS realization law and governed RMS lowering from two distinct input boundaries `[0, 1]` to the accepted one-boundary alias subject `[0, 0]`; preserve the existing distinct-operand realization byte-for-byte except for authority provenance that must move.
- Emit fold sources `[Occurrence(0)]` and pass sources `[Occurrence(0), Intermediate(0)]`. Use one dense pass leaf twice in the accepted `weight * (value * root)` scalar expression.
- Retain exactly four ordered operand bindings and three pass buffer bindings for the alias case. Do not add an operand-to-access map or a KIR/artifact buffer-coalescing rule.
- Move only the RMS realization-law registration to revision 2 and only the RMS lowering capability to revision 2. Move the governed physical provider to revision 2 because its proposal population changes; do not raise unrelated law or capability rows merely because their current implementation shares a constant.
- Recompute and enumerate every request, registry, proposal, selected-provider, executable, artifact, cache, and explain value that actually moves. Preserve existing identity domains and manifest schema unless exact encoder evidence contradicts the accepted decision audit.
- Retire the now-redundant `value_input == weight_input` physical guard only after both law and lowering admit the subject and end-to-end construction proves the canonical schedule.

## Required controls

Cover the complete evidence and independent subject perturbations listed in the prerequisite decision. In particular, prove distinct operands keep their existing three-read pass; one declared input read densely and through a different relation remains two accesses; changing the scalar expression's second use changes results; and each of the three revision subjects is load-bearing independently.

## Unsupported population

No general alias analysis, per-use access policy, mapped-read coalescing, different-shape alias, in-place mutation, runtime buffer alias promise, global input deduplication, or second staged family is admitted. Exact same semantic value plus identical dense relation is the whole slice.

## Closes when

The accepted alias occurrence compiles and evaluates bit-for-bit, the exact retained populations and revision/identity movements are pinned, every negative control fails for its own subject, distinct-operand behavior remains exact, package and workspace gates pass, and the support boundary is stated without implying broader alias admission.
