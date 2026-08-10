---
id: scope-the-remaining-elementwise-float-algebra-families
title: Scope the remaining elementwise float algebra families
status: deferred
priority: p2
dependencies: []
related: [scope-the-fused-multiply-add-semantic-family, select-the-first-general-elementary-function-keys, scope-the-standalone-extrema-and-clamp-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, pointwise, deferred]
---
## User-visible outcome

`Subtract`, `Divide`, and the unary algebraic operations of F-05 (`abs`, `negate`, `sign`) reach the same delivered state `tiler::add-f32@1` and `tiler::multiply-f32@1` already have, so that a frontend lowering that needs a subtraction, division, or unary algebraic step stops being blocked on families whose rounding is already fixed by [ADR 0024](../docs/decisions/0024-initial-arithmetic-rounding.md) (RN-ties-to-even for `Add`/`Subtract`/`Multiply`/`Divide`; implementation still `partial`) while signed-zero and NaN statements remain owed separately.

## Why this is deferred rather than open, and what this track is *not*

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md) splits elementwise float arithmetic by arity and by numerical obligation: F-05 unary float arithmetic is one operand and one result, "exactly rounded per [ADR 0024](../docs/decisions/0024-initial-arithmetic-rounding.md) where the operation is algebraic", with `abs`, `negate`, and `sign` exact and owing separate signed-zero and NaN statements; F-06 binary float arithmetic is two operands of one identical resolved type with **no ambient promotion or autocast**, "separate rounding per operation", and `divide` owing its zero-divisor and inexactness behaviour.

**Fact — two of F-06's operations are delivered and the rest of both families are not.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) records `constant-f32`, `add-f32`, and `multiply-f32` at R6 with a bounded R7 execution row. Remaining undelivered pointwise float algebra sits on a dedicated matrix row at R2 naming `Subtract`, `Divide`, and negation, with evidence that no **semantic** standard-registry key for those three exists; required `Fma` is a **separate** R2 row (split out on 2026-08-05). This ticket owns the remaining-algebra row / delivery track **O-17** (F-05 whole plus F-06 `subtract`/`divide`); `Fma` has its own track and ticket, because a single-rounding fused operation is a different correctness argument and a different physical precondition. A governed **scalar** key `tiler.scalar::divide-f32@1` exists for composite index-region bodies and is not a general semantic `Divide` admission.

**Inference — remaining algebra is one track (O-17).** They share the exact-rational-then-one-rounding oracle shape, the equal-shape rank rule with the narrow rank-zero scalar admission, the refusal of mixed precision by name, and the same minimum physical route: [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) places F-05 and F-06 together in its *covered — direct scalar or map route* class, where "the profile must have a serial or direct kernel, and refusing one is a defect". Track coverage is full F-05 (`abs`, `negate`, `sign`) plus F-06 `subtract` and `divide`; the matrix row title names only negation among the unaries, but O-17 and the taxonomy do not drop `abs`/`sign`. `Divide` adds exactly one thing the others do not owe — a zero-divisor rule and the reciprocal rewrite permission governed by the `reciprocal_transform` numerical dimension in [numerical semantics](../docs/numerical-semantics.md) — and that is an added field rather than a second correctness argument. A general semantic `Divide` key cannot inherit that permission from silu, rms_norm, or softmax: those composites pin divide-versus-reciprocal-multiply inside their own formula contracts, and a pinned formula is not an exercise of reciprocal rewrite permission.

**Correction — 2026-08-10.** Earlier wording on this ticket claimed `Subtract`/`Divide`/negation/`Fma` shared one matrix row whose evidence named Fma, and that `Divide`'s reciprocal permission was held under closed **Q-SEM-001** ("Q-SEM-004's sibling"). The matrix split Fma out on 2026-08-05; Q-SEM-001 closed as numerical-policy-presets supersession and is not a reciprocal-substitution question. Live reciprocal authority is `reciprocal_transform` / rewrite permission for a general `Divide` key, with explicit non-inheritance from composite formula pins. The unsubstantiated "four accepted ADRs already describe" count is dropped; ADR 0024 is the named rounding ADR. Optional precision: "no key" means no semantic standard-registry OpKey (scalar `divide-f32` still exists).

## Activation trigger

A named workload or frontend lowering needs one of the undelivered O-17 operations (`Subtract`, `Divide`, or any of F-05 `abs`/`negate`/`sign`), stated as the matrix row's own trigger states it in substance: each entering operation requires a key, an evaluator, a fusion role, and a backend realization before it may be claimed above R2; `Divide` additionally needs its reciprocal rewrite permission resolved under the `reciprocal_transform` dimension rather than assumed from a composite pin. **Note — 2026-08-10.** The matrix trigger cell still says "resolved under Q-SEM-001"; that citation is stale (Q-SEM-001 closed); residual product debt is a docs/roadmap matrix cell repair, not a status change on this ticket.

## What the work would be, when it starts

Per entering operation: the key and its schema, the exact-rational evaluator with one rounding at materialization, the signed-zero and NaN statements that ADR 0024 does not supply because it fixes rounding and not special values, the fusion role, the `ScalarProgram` spelling, and the backend emission — plus, for `Divide` alone, the zero-divisor result and the reciprocal-substitution / rewrite permission under `reciprocal_transform`, which must be resolved rather than assumed because two delivered families already sit on opposite sides of it: the activation and normalization pin a division and withhold the permission to become a reciprocal multiply, and the softmax pins the multiply and withholds the permission to become a division.

## Explicit non-goals

- `Fma`, which is [`scope-the-fused-multiply-add-semantic-family`](scope-the-fused-multiply-add-semantic-family.md)'s and is not a composition of a multiply and an add.
- Extrema and clamp, which are [`scope-the-standalone-extrema-and-clamp-families`](scope-the-standalone-extrema-and-clamp-families.md)'s under ADR 0023.
- Transcendentals, which are [`select-the-first-general-elementary-function-keys`](select-the-first-general-elementary-function-keys.md)'s: an algebraic operation owes exact rounding and no accuracy contract, and conflating the two is what the taxonomy's D5 independence rule forbids.
- Any implicit promotion. A mixed-precision application is refused by name, which is on the taxonomy's intentionally-invalid list and must never become a ticket.

## Closes when

Each undelivered O-17 member has a semantic key, an evaluator, a fusion role, and a backend realization, or is explicitly recorded as unneeded by any named consumer; and `Divide`'s reciprocal rewrite permission is resolved under `reciprocal_transform` rather than inherited from either delivered composite precedent.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-17** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-05 and the undelivered half of F-06 and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named workload needs a standalone subtraction, division, or negation: the pinned workload's subtractions and divisions are all *embedded* in registered composite families — the softmax's maximum subtraction and reciprocal multiply, the activation's division — each pinned inside its own key's normative reference rather than expressed as a general operation. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — historical census at that date reported 46 governed keys comprising dtype identities, the ULP metric key, and eighteen registered operation keys; the family's semantic keys were absent. Treat that integer census as historical, not a pin: later families widened the population; the load-bearing claim is absence of subtract/divide/negate (and abs/sign) semantic keys.
- 2026-08-09 — **not fired.** The staged elementary/composite vocabulary has widened, but no standalone subtract, divide, or negate key exists and no frontend lowering requires one. Subtraction/division inside softmax, RMS normalization, and activation remain governed parts of those composite keys rather than general algebra admissions.
- 2026-08-10 — **not fired (prose recheck; trigger text repaired).** No named workload still requires standalone O-17 semantic keys. F-05 remains fully undelivered (`abs`, `negate`, `sign` as well as the binary half). Scalar `tiler.scalar::divide-f32@1` continues to serve composite bodies only. Recheck absence of semantic OpKey constructors for subtract/divide/negate/abs/sign under `crates/tiler-ir/src/semantic/` (standard registry + `standard_operations`); do not treat the 2026-08-05 key-count integers as current.
