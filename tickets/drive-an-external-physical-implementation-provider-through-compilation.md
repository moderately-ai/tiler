---
id: drive-an-external-physical-implementation-provider-through-compilation
title: Drive an external physical implementation provider through compilation
status: todo
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary]
related: [prototype-complete-physical-plan-selection, wire-capability-and-refinement-into-compile-path]
scopes: [implementation/compiler, implementation/ir, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, compiler]
---
## User-visible outcome

An out-of-crate caller can install a physical implementation provider into the ordinary compiler session, have its candidates reverified and considered additively, and observe exact selected-provider provenance in the resulting plan and explain output.

## Implementation keys

- Promote only the exact physical-provider facade accepted by the composition ADR.
- Let providers propose bodies, applicability, and estimates through bounded writers; derive provider identity from registration and derive resource/boundary facts from verified output.
- Retain several valid providers' implementations side by side for cost-based selection.
- Preserve the asymmetry with lowering: two lowering authorities for one occurrence are ambiguous, while two physical implementations of one verified region are alternatives.
- Reject malformed provider output as a provider/compiler defect rather than silently treating it as an empty offer.
- Keep empty offer, hard rejection, unknown analysis, provider defect, and cost disadvantage distinct in explain output.
- Add an out-of-crate compile fixture and perturb installation, identity, region coverage, target applicability, and verifier bypass attempts.
- Review the exact public trait/module/session boundary with Tom before acceptance.

## Closes when

An external provider reaches `enumerate_frontier` through `session::compile`, the selected plan records its non-forgeable identity, every negative control fails for the intended reason, targeted nextest and Clippy pass, and one final `make full` passes for the batch.

## Graph maintenance

- Unblock payload production and final provider composition only through the accepted public seam.
- Keep semantic-equivalence trust limitations explicit; structural verification cannot prove arbitrary replacement mathematics.
- Update ADR 0078's implementation status and governed seam inventory only after the path is genuinely external and exercised.
