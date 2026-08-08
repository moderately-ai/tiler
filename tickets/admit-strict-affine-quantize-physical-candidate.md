---
id: admit-strict-affine-quantize-physical-candidate
title: Admit strict-affine Quantize as a committed physical candidate
status: closed
priority: p2
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/reference, implementation/metal, implementation/build, implementation/runtime, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, physical-candidate, metal]
closed_reason: obsolete
closed_note: The selected workload has no Quantize or Assemble producer; its direct encoded input is enforced through resolved-value conformance and the fused contraction consumer.
---

## Obsolete outcome — 2026-08-08

No physical `Quantize` candidate is implemented by this ticket. Its own activation rule required [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) to select a strict-affine `Quantize` producer with a real downstream consumer and required closure as obsolete otherwise.

The completed selection instead chose role-addressed strict-affine U8 **interface inputs** consumed by `DequantizeStrictAffine` fused into a contraction. The executed program contains no `Quantize` or `Assemble`. Keeping this ticket as the enforcement chain's synthetic consumer would therefore impose internal compound grouping and operation-precondition work on a workload that exercises neither.

Before closure, every graph record that had named this ticket or its obsolete route was repaired:

- [`carry-semantic-enforcement-plans-through-program-and-artifact`](carry-semantic-enforcement-plans-through-program-and-artifact.md) now derives a static plan from the delivered direct-binding conformance contract and protects the first fused contraction consumer.
- [`implement-first-runtime-semantic-value-precondition-enforcement`](implement-first-runtime-semantic-value-precondition-enforcement.md) now executes that direct-input conformance after `RoutingCommit` and before result work.
- [`implement-first-quantized-backend-profile`](implement-first-quantized-backend-profile.md) consumes the corrected runtime vertical without a direct or transitive internal-grouping prerequisite.
- The separate [`group-internal-compound-materializations-by-logical-value`](group-internal-compound-materializations-by-logical-value.md) capability is deferred until an actually selected internal producer fires its named trigger; it was not a dependent of this candidate.

Do not reopen this ticket as a placeholder for activation quantization, requantization, or another imagined producer. A future selected internal producer must receive a new bounded candidate ticket derived from its exact producer, consumer, profile, and evidence.
