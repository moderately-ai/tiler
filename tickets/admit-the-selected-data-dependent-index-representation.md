---
id: admit-the-selected-data-dependent-index-representation
title: Admit the selected data-dependent index representation
status: blocked
priority: p1
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, decide-the-data-dependent-index-representation-public-surface]
related: [revise-adr-0108-with-a-complete-data-dependent-index-vertical, admit-an-invocation-scoped-gather-index-validation-receipt, emit-the-indirect-gather-on-metal]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, verification, identity, decision, needs-tom, public-boundary]
---
## User-visible outcome

The representation ADR 0108 ultimately accepts is admitted as a complete verified logical index form, while every existing direct-access byte and verifier guarantee remains unchanged.

## Required boundary

- Implement only the accepted nested-read or tagged-access form; do not blend the candidates.
- Carry the outer coordinate, nested source tensor, complete source coordinates, U32 value semantics, rank and reachability checks, exact bounds obligation, compaction/remapping, alpha-equivalence, canonical ordering, encoding, views, errors, reference evaluation, compiler recognition, and explanation as one coherent population.
- Preserve all old canonical bytes and pin every identity-domain step the accepted ADR requires.
- Retain the gather bound as either a static proof or one exact mandatory invocation-validation obligation. This ticket does not mint a runtime receipt and cannot treat an obligation as discharged.
- Keep direct access verification and ADR 0046 unchanged; scatter and data-dependent output shapes remain absent.

## Closes when

The selected form is constructed and inspected through the reviewed surface, all exhaustive consumers are updated, static proof reaches executable coverage, the dynamic form remains pending on the named receipt, subject perturbations independently fail, and targeted plus full gates pass.
