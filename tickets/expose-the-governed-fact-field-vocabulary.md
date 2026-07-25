---
id: expose-the-governed-fact-field-vocabulary
title: Expose the governed fact-field vocabulary that facts() readers need
status: todo
priority: p2
dependencies: []
related: [declare-governed-scalar-numerical-facts]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, numerics, public-surface]
---
Both IR layers publish their numerical facts as readable canonical records while keeping private the field identifiers a reader needs to interpret them. A consumer can obtain the record and cannot name a field in it without hardcoding an integer.

**Fact (inspected source, base `f286289` plus `declare-governed-scalar-numerical-facts`).** `ScalarOperationDefinition::facts()` and `ScalarOperationDefinition::conformance()` in `crates/tiler-ir/src/index/scalar.rs` are `pub` and return `&CanonicalValue`. The field IDs that give those records meaning — `SCALAR_FACT_ROUNDING`, `SCALAR_FACT_NAN_RESULT_RULE`, `SCALAR_FACT_CANONICAL_NAN_BITS`, `SCALAR_FACT_CONTRACTION_PERMITTED` — are private module constants. The semantic layer has the same shape: `OperationDefinition::canonical_facts()` is `pub`, and `crates/tiler-ir/src/semantic/registry.rs` writes its field IDs as bare `AttributeFieldId::new(1)`/`new(2)`/`new(3)` literals inside the private `arithmetic_f32_facts` and `standard_conformance` functions, with no named constant at all.

**Inference — the gap is real but symmetric, which is why it was not closed opportunistically.** `declare-governed-scalar-numerical-facts` populated the scalar records and deliberately matched the semantic layer's existing privacy rather than exposing one layer's vocabulary and not the other's. Publishing field identifiers is a new public surface and therefore owner-reserved, and the two layers should be decided together: exposing only the scalar constants would assert that the scalar record is the interpretable one, which is not a distinction anything has decided.

**The consequence to weigh.** An out-of-crate reference capability or index-access lowering provider is exactly the consumer these facts were declared *for* — it is supposed to read them and conform. Today it must either hardcode `AttributeFieldId::new(2)` against a record whose numbering no contract states, or ignore the facts and rely on the prose in `NormativeDefinitionRef`. The first is a silent-breakage hazard the moment a field is renumbered; the second is the situation the facts were added to end.

**What closes this.** One decision, applied to both layers together:

1. Decide whether the fact-field vocabulary is public API. If it is, name the constants at both layers and document each field's meaning and the exact conditions under which it is present or absent — for the scalar layer, that fields 1 and 2 are stated by every governed definition while 3 and 4 are stated only where defined.
2. Decide whether the semantic layer's operation-local numbering is intended. `arithmetic_f32_facts` puts the canonical NaN payload at field 2 and `strict-serial-sum-f32` puts it at field 3; a published vocabulary either has to state that field IDs are record-local, or normalize them.
3. If the vocabulary stays private, say so in the accessor documentation, so a consumer learns from the API that the record is opaque to it rather than discovering it by reading the crate.

Requires an owner decision on the public boundary before implementation; ADR 0075 scopes public-boundary approval by change category.

## Decision — Tom, 2026-07-25

**Approved: promote.** ADR 0075 reserves public-surface promotions to the owner; this one is granted.

`facts()` is publicly readable while its field IDs are private at both layers, so a reader can obtain facts it cannot interpret. Publishing the vocabulary makes it a durable identity surface: state that renumbering is thereafter a breaking change, so nobody treats the IDs as internal later.
