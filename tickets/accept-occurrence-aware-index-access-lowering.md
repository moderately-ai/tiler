---
id: accept-occurrence-aware-index-access-lowering
title: Accept the occurrence-aware index-access lowering context
status: awaiting-decision
priority: p0
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, capability, api-boundary, milestone-0b]
---
`wire-capability-and-refinement-into-compile-path` made two breaking changes to public signatures in `pub mod capability` and `pub mod legality`. Under ADR 0075 a breaking change to an existing public signature always requires Tom's review before merge, so the branch carries them as a concrete draft and this ticket holds the decision.

**The two changes.** `capability::IndexAccessLoweringContext::new` gained a second parameter, a new `capability::LoweredOccurrence` carrying the occurrence's distinct input boundaries, the ordered operand-to-input mapping, the ordered result boundaries, and the host-canonical `OperationAttributes`. `legality::SemanticOccurrence::new` gained an `attributes` parameter so refinement can hand those attributes through and bind them into the reusable content identity.

**Why the design needs them.** A registered index-access capability is keyed by `(family, operation, signature, provider)`, and `LoweringSignature` carries resolved value types only — no shapes and no attributes. A shape- and attribute-blind provider can therefore emit exactly one fixed region, so a governed family would need one registered provider per program shape *and per attribute value*. The standard fixture already breaks that: it has two `tiler.constant-f32` occurrences with different bit patterns, and two providers registered for one key are an unresolvable `AmbiguousCapability` rather than two lowerings. The asymmetry is also an oversight in the draft — `ScalarLoweringContext` already exposes operands, attributes, and signature.

**Alternatives considered.** (a) Keep `new(builder)` and add `new_for_occurrence(builder, facts)` with `Option`-returning accessors: additive, but it models an always-present fact as optional and lets a provider silently misbehave on the `None` path. (b) Build the lowering registry per compilation request with shapes and constant bits baked into provider instances: keeps the public signature, makes the "governed" registry program-specific, and still cannot separate two constants of one family — it only moves the ambiguity. (c) The accepted draft: the context always carries the occurrence, because a provider is always invoked to lower a specific one.

**Closing evidence.** Tom accepts, amends, or rejects the shape. On acceptance this ticket is closed by the merge of the branch; on rejection the branch returns to alternative (a) or (b) and the governed provider set is reshaped accordingly.
