---
id: revisit-kernel-lowering-placement
title: Revisit whether the canonical kernel lowering belongs in tiler-ir
status: done
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

**The original trigger has fired.** `tiler-metal` now consumes the canonical
kernel layer, and the compiler calls `lower_scheduled_region` from its physical
planning path. Review those concrete call sites for evidence that
backend-specific lowering wants to sit beside the canonical derivation or that
a second lowering has accumulated in `tiler-ir`.

## User-visible outcome

All backends consume one verified canonical kernel meaning, without duplicating
the derivation or making `tiler-ir` an accidental home for unrelated compiler
passes. Keep the present placement if the landed backend shows no real layering
cost; movement needs concrete evidence and must preserve one authority.

If it does move, the derive-and-compare gate must move or be restructured with
it; do not leave a copy of the canonical derivation behind, since two derivations
that can drift apart is a worse outcome than either placement.

## Closes when

The landed Metal and compiler call paths are inspected, the placement is either
affirmed against that evidence or changed without duplicating the canonical
derivation, and the architectural rationale is recorded.

## Outcome

The trigger was evaluated against the landed Metal backend. Metal consumes the
existing canonical lowering without introducing a neighboring
backend-specific lowering, the compiler calls the same authority, and no second
verifier-owned lowering exists in `tiler-ir`. The approved placement remains
supported. Reopen placement only if a second canonical lowering appears or a
concrete dependency-direction conflict is measured.
