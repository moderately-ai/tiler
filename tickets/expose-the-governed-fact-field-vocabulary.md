---
id: expose-the-governed-fact-field-vocabulary
title: Expose the governed fact-field vocabulary that facts() readers need
status: done
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

## Approved implementation outcome

Publish the field names in the namespace of the record they interpret, document
presence and absence rules, and state that field IDs are record-local unless a
shared vocabulary is explicitly introduced. Renumbering a published ID is a
breaking identity change. Do not normalize equal integer values across
different records merely because their storage shape matches.

## Decision — Tom, 2026-07-25

**Approved: promote.** ADR 0075 reserves public-surface promotions to the owner; this one is granted.

`facts()` is publicly readable while its field IDs are private at both layers, so a reader can obtain facts it cannot interpret. Publishing the vocabulary makes it a durable identity surface: state that renumbering is thereafter a breaking change, so nobody treats the IDs as internal later.

## Outcome (2026-07-27)

Both layers published together, which is what the ticket required — exposing one would have asserted that its record is the interpretable one, a distinction nothing decided.

**Semantic layer** (`tiler_ir::semantic`), eleven constants, each naming a field of *one* record: `F32_TYPE_FACT_CLASS`/`_WIDTH_BITS`; `CONSTANT_F32_FACT_PAYLOAD_RULE`; `ARITHMETIC_F32_FACT_ROUNDING`/`_CANONICAL_NAN_BITS`/`_CONTRACTION_PERMITTED`; `SERIAL_SUM_F32_FACT_FOLD_ORDER`/`_ACCUMULATION`/`_CANONICAL_NAN_BITS`; `CONFORMANCE_FACT_IDENTITY`/`_VERSION`. Every construction site in `registry.rs` now uses them instead of a bare `AttributeFieldId::new(N)`, so a name and the record it describes cannot drift apart.

**Scalar layer** (`tiler_ir::index`): the four `SCALAR_FACT_*` constants and the two profile strings a consumer must compare against — `CANONICAL_ARITHMETIC_NAN_PROFILE` and `DECLARED_PAYLOAD_PRESERVED` — promoted from private. They already carried the presence and absence rules in their documentation; publishing them makes those rules readable by the consumer they were written for.

**Record-local numbering is stated at both sites and tested.** The pair that proves it is contraction: field **3** on the semantic arithmetic record and field **4** on the scalar one. Reading either number against the other record answers a different question, and `the_published_scalar_fact_fields_read_the_governed_records` asserts the two constants differ so a later "cleanup" that normalized them would fail rather than silently repoint every conforming reader.

**Renumbering is now a breaking identity change**, stated in both vocabulary blocks, per Tom's decision that publishing makes this a durable identity surface.

**One field's meaning was derived rather than read.** The semantic arithmetic record's field 3 is a bare `boolean(false)` with no name at its construction site. It is contraction: the normative references for the standard multiply and add say "separate binary32 multiply" and "separate binary32 addition", and the scalar layer mirrors the same concept as an explicit contraction fact. The constant is named accordingly and the derivation is recorded on it.

### Measurement boundary

The test reads the **scalar** records through `FrozenScalarRegistry::definition`. It does not read the semantic ones, because `FrozenSemanticRegistry` exposes no accessor returning a registered `OperationDefinition` or value-type definition — there is no `value_type()` or `operation()` on it. The semantic constants are therefore proven correct at their *construction* sites, where they are now used, and not at a read site, because this crate offers no public read path to exercise. **That is worth naming rather than glossing:** this ticket's premise is that "`facts()` is publicly readable while its field IDs are private", and for the semantic layer the readable half appears to be reachable only to a provider holding a definition it built. Whether a consumer can obtain a governed semantic definition at all is a separate question this ticket did not settle.

**Closed the same day** by `decide-whether-governed-semantic-definitions-are-readable-out-of-crate`: `FrozenSemanticRegistry` now offers `operation_facts` and `value_type_facts`, and the test reads both layers.
