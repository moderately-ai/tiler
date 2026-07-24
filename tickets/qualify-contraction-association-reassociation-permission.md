---
id: qualify-contraction-association-reassociation-permission
title: Qualify contraction-order exploration with a reassociation permission
status: in-progress
priority: p2
dependencies: []
related: [scope-einsum-contraction-support, settle-contraction-chain-distributivity-permission, reconcile-dtype-cast-enforcer-with-boundary-properties]
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

## Outcome

The asymmetry was an oversight, not an intentional exemption. All four logical-exploration rules entered in the initial commit `9acca0d` with wording unchanged since (`git log -S` on each rule string over `docs/compiler/optimizer.md` returns only `9acca0d`), so no edit ever added or removed a qualifier. Two readings that would have made it intentional were checked and refuted: the trailing "reassociate arithmetic or reductions" item is a rule with its own subject rather than a scope statement over the list — the normalization list and the physical-implementation list each carry their scope statement as separate prose after the list, and the logical-exploration list carried none — and the association rule is placed in logical exploration and listed among equivalent expressions, so it is not a physical choice exempt from semantic-order authority.

The document also lacked the general obligation, so the fix closes the class rather than one instance:

- the third rule now reads "choose alternative associations of a tensor contraction only when the effective reassociation permission authorizes the regrouping";
- a following paragraph states that each rule names the permission it consumes (ADR 0011) and that a rule naming none consumes none, with the view-pushing rule's exemption derived from ADR 0020's value-only floating-point exceptions;
- a second paragraph records that a reassociation permission is necessary but not established as sufficient, because a chain regroup redistributes products across sums rather than only regrouping one reduction's contributors, and fails the rule closed until `settle-contraction-chain-distributivity-permission` settles it;
- a third paragraph names `StrictF32NumericalContract::governed` and separates its `reassociation` field from ADR 0015's `contraction` field.

The equivalent-expressions example at the former line 159 needed the same treatment for a sharper reason: the document defines logical equivalence as computing the same tensor *under a stated numerical policy*, so listing contraction associations unconditionally asserted an equivalence group that does not exist under the registered contract. It is now qualified, with a sentence recording that logical equivalence is policy-relative. `docs/compiler/fusion-and-scheduling.md` gained the matching note under "Future contraction schedules": the global contraction-order choice is the same authorized rewrite rather than a schedule freedom.

No other rule in the logical-exploration list has the gap. Checked and clear: the normalization list is covered by "Normalization must not silently change floating-point evaluation order"; the physical-implementation list, including "serial, subgroup, threadgroup, or multi-pass reduction" and "direct or GEMM-backed contraction", is covered by the stronger requirement that each candidate's machine-checkable numerical guarantee refine every effective operation contract. One adjacent finding is deferred to `reconcile-dtype-cast-enforcer-with-boundary-properties`: "dtype cast" is listed as an enforcer while dtype is absent from the boundary-contract list, whose paragraph states that numerical policy is not a schedule-supplied property.
