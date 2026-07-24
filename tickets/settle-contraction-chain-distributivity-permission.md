---
id: settle-contraction-chain-distributivity-permission
title: Settle whether reassociation permission authorizes contraction-chain regrouping
status: todo
priority: p2
dependencies: []
related: [qualify-contraction-association-reassociation-permission, scope-einsum-contraction-support, disambiguate-contraction-in-the-glossary]
scopes: [contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics, optimizer]
---
`docs/compiler/optimizer.md` now admits the tensor-contraction association rewrite only under an effective reassociation permission, and the Milestone 6 framing in `docs/roadmap.md` calls regrouping `(AB)C` into `A(BC)` a floating-point reassociation in ADR 0014's sense. That makes the reassociation permission necessary. It does not establish that it is sufficient.

`docs/numerical-semantics.md` defines reassociation as changing grouping while preserving logical operand order, and states the reduction-order contract over one reduction's contributors. A contraction-chain regroup is stronger than that. For output `[i, l]`, `(AB)C` forms the rounded partial `T[i, k] = sum over j of A[i, j] * B[j, k]` and then multiplies `T[i, k]` by `C[k, l]`; `A(BC)` never forms those products at all. The two agree over the reals by distributivity, which floating-point multiplication does not satisfy, so the rewrite redistributes products across sums rather than only regrouping one sum's contributors.

Decide whether the reassociation permission covers this, or whether a contraction chain declares a distinct algebraic capability and consumes a distinct permission alongside it. `docs/numerical-semantics.md` is the normative owner and would carry the answer; the logical-exploration rule in `docs/compiler/optimizer.md` and the future contraction schedules in `docs/compiler/fusion-and-scheduling.md` cite it and would be updated on closure. Until then the optimizer rule fails closed rather than assuming the weaker permission covers it.

Nothing observable depends on this today: `StrictF32NumericalContract::governed` in `crates/tiler-compiler/src/request.rs` is the only numerical contract the compiler registers and sets `reassociation` to `NumericalPermission::Forbidden`. The question becomes reachable when a contract permitting reassociation is registered, or when the semantic half of Q-SEM-015 admits a tensor-contraction family, whichever comes first.

"Contraction" is the tensor sense throughout — summation over indices shared by two or more operands — and never ADR 0015's fused-multiply-add permission.

Found by `qualify-contraction-association-reassociation-permission`, which owns the optimizer rule but does not hold `contracts/numerics`.
