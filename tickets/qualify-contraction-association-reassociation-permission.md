---
id: qualify-contraction-association-reassociation-permission
title: Qualify contraction-order exploration with a reassociation permission
status: in-progress
priority: p2
dependencies: []
related: [scope-einsum-contraction-support]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, numerics]
claimed_from: todo
assignee: agent-qualify-contraction-association-reassociation-permission
lease_expires_at: 1784923183
---
The `ExploreLogicalAlternatives` stage in `docs/compiler/optimizer.md` "adds only proved
contract-preserving forms". Its logical-exploration rule list contains "choose
alternative contraction associations" (tensor sense) immediately above
"reassociate arithmetic or reductions only when numerical policy permits" — but
the contraction bullet carries no such qualifier.

Regrouping a contraction chain from `(AB)C` to `A(BC)` changes which partial sums
are formed and rounded, so it is a floating-point reassociation in ADR 0014's
sense and requires both a reassociation capability and an effective numerical
permission. Under the strict `f32` contract the first profile registers
(`StrictF32NumericalContract::governed` sets `reassociation:
NumericalPermission::Forbidden`) the rewrite is illegal, not merely unexplored.

Add the qualifier to that bullet, or state explicitly why contraction
association is exempt. `docs/compiler/fusion-and-scheduling.md` line "Einsum adds
global contraction-order choices" may need the matching note. Check whether the
same gap exists for any other bullet in that list.

Found while writing the Milestone 6 contraction framing
(`scope-einsum-contraction-support`), which does not own `contracts/optimizer`
and therefore records the finding rather than fixing it. Note that "contraction"
has two unrelated senses in this corpus; this ticket is about the tensor sense
in the optimizer contract, and about ADR 0015's FMA-permission sense only as the
permission that governs it.
