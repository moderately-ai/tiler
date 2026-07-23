---
id: own-operation-family-support-matrix
title: Own the operation-family support matrix (breadth tracking)
status: todo
priority: p1
dependencies: []
related: [enumerate-the-mature-tensor-dtype-taxonomy, scope-einsum-contraction-support]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, breadth, epic]
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
