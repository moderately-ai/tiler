---
id: own-operation-family-support-matrix
title: Own the operation-family support matrix (breadth tracking)
status: in-progress
priority: p1
dependencies: []
related: [enumerate-the-mature-tensor-dtype-taxonomy, scope-einsum-contraction-support]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, breadth, epic]
claimed_from: todo
assignee: agent-own-operation-family-support-matrix
lease_expires_at: 1784917695
---
Wide operation support is the long-term goal, but it currently has no single
durable owner in the work graph. The dtype axis is owned:
`enumerate-the-mature-tensor-dtype-taxonomy` (done) enumerates the dtype universe,
though it deliberately claims no reference/optimizer/backend support. The
**operation** axis has no equivalent owner, and no artifact tracks
recognized-versus-supported state, so the surface can stay implicitly narrow (the
first profile is 4 strict-f32 operations) while breadth silently falls off the
roadmap.

Add one legible owner — an `docs/open-questions.md` entry or a `docs/roadmap.md`
support-matrix section — that enumerates the operation families and their current
maturity state (type-system reservation / recognized identity / reference-evaluated
/ optimizer-supported / backend-realized), cross-referencing the dtype taxonomy for
the dtype axis. At minimum enumerate: pointwise transcendentals (accuracy contracts
accepted per ADR 0016/0042, but no operation defined and no implementing ticket),
integer arithmetic and division families, cast/convert families, reduced-precision
float arithmetic (f16/bf16/fp8/fp6/fp4 — recognized identities only), reductions
beyond strict sum, and tensor contraction / matmul / einsum (owned separately by
`scope-einsum-contraction-support`). Give each a reconsideration trigger.

Purpose is visibility and tracking, not authorization: this ticket makes the
breadth surface legible so it can be scheduled deliberately. It does not itself
authorize implementing any operation, and it must not overstate current support.
Touching `docs/operation-extensions.md` or `docs/numerical-semantics.md` requires
adding `contracts/foundation` / `contracts/numerics` before starting.

## Outcome

The [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) is the durable owner, and [Q-SEM-014](../docs/open-questions.md) is its entry in the question index. The roadmap holds the matrix because `docs/open-questions.md` states that each of its entries is one question with one owner and one closure or trigger; fifteen families with per-family rungs and per-family triggers are not one question, and splitting them into fifteen entries would destroy the cross-family comparison that is the point. The roadmap already carries `roadmap_status: proposed` and already describes progression rather than delivered support, which is exactly the framing a visibility artefact needs. Q-SEM-014 makes the matrix reachable from the durable question index and states the complement of Q-SEM-003: that question covers admitted tuples, this one covers unadmitted families. [Design map](../docs/design-map.md) gains one navigation row so the question "which operation families are actually supported" resolves to an owner.

The ladder keeps `AGENTS.md`'s four maturity claims distinct and decomposes only implemented support: R1 type-system reservation, R2 architectural seam, R3 recognized identity, R4 reference-evaluated, R5 optimizer-supported, R6 backend-realized, R7 tested guarantee. R3 through R6 are the four layers of implemented support; R7 is scoped to an operation, dtype, target, and layer, never to a family.

Fifteen families are enumerated, every one with an explicit trigger. Three reach R6 — the `f32` constant, `Add`, `Multiply`, and strict serial `Sum` of the first profile — and every rung is justified from inspected source or an accepted ADR's own recorded status rather than from intent. Three absence claims carry the exact reproducible command that establishes them. Two structural limits that would otherwise inflate R5 and R6 are recorded: the compilation request path recognizes exactly one program shape, and the lowering-capability registry has no production caller. Both are owned by the optimizer conformance gate.

Two source claims were corrected during the work after a broader search contradicted a first reading, and both corrections are recorded in the matrix: contraction has real tensor-sense mentions in the optimizer and fusion contracts rather than only ADR 0015's unrelated FMA sense, and `Select` names four different constructs across the corpus. The second produced a follow-up ticket, [`disambiguate-select-across-ir-layers`](disambiguate-select-across-ir-layers.md), alongside [`reconcile-illustrative-operation-names-with-governed-keys`](reconcile-illustrative-operation-names-with-governed-keys.md) for the IR contract's illustrative operation spellings.

No `contracts/foundation` or `contracts/numerics` file was edited; both were read as evidence and cross-referenced only.
