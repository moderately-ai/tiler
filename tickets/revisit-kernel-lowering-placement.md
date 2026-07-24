---
id: revisit-kernel-lowering-placement
title: Revisit whether the canonical kernel lowering belongs in tiler-ir
status: todo
priority: p2
dependencies: []
related: [prototype-structured-kir-slice]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, architecture]
---
`tiler_ir::kernel::lower_scheduled_region` — the canonical schedule-to-kernel
lowering — lives in `tiler-ir`, which otherwise holds representation and
verification rather than lowering. Tom reviewed this placement and **approved it
as-is** on 2026-07-24; this ticket exists so the tension is revisitable rather
than forgotten, not because the decision was wrong.

The reasoning that justified it: the kernel verifier's final gate is
derive-and-compare — it re-derives the canonical body and requires structural
equality — so the canonical derivation must exist inside the verifier regardless.
Publishing it as `lower_scheduled_region` therefore exposes the authority that
already exists instead of creating a second, hidden one. A compiler-side lowering
would either duplicate that derivation (two authorities that can silently
diverge) or force the refinement gate to be restructured.

The counter-tension: ADR 0070 places shared compiler IR in `tiler-ir`, and a
lowering is arguably a compiler concern. If more lowerings accumulate here, the
crate's role blurs from "representation and verification" to "representation,
verification, and some lowerings", which is exactly the kind of drift that is
cheap to prevent and expensive to unwind.

**Trigger for reconsideration:** when the Metal backend lands and we can observe
whether the layering actually chafes — concretely, whether backend-specific
lowering wants to sit near this one, or whether a second lowering has appeared in
`tiler-ir` for the same "the verifier needs it anyway" reason. Decide then with
evidence rather than now by argument.

If it does move, the derive-and-compare gate must move or be restructured with
it; do not leave a copy of the canonical derivation behind, since two derivations
that can drift apart is a worse outcome than either placement.
