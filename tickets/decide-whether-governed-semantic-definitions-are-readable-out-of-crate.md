---
id: decide-whether-governed-semantic-definitions-are-readable-out-of-crate
title: Decide whether governed semantic definitions are readable out of crate
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [api]
---

Found while closing `expose-the-governed-fact-field-vocabulary`; read that ticket's Outcome section first.

## The question

That ticket's premise was that "`facts()` is publicly readable while its field IDs are private at both layers, so a reader can obtain facts it cannot interpret". The field IDs are now published. **For the semantic layer, the first half of that premise is unverified.**

**Fact.** `FrozenSemanticRegistry` exposes no accessor returning a registered `OperationDefinition` or `ValueTypeDefinition`. Exact check: `grep -n "pub fn " crates/tiler-ir/src/semantic/registry.rs` lists `resolve_marker`, `validate_type`, `project_operation_authority`, `project_operation_occurrence_authority`, and the registrar's `register_*`, and none of them returns a definition whose `facts()` a consumer could call. The scalar side does have one — `FrozenScalarRegistry::definition` — which is why the vocabulary test reads scalar records and not semantic ones.

**Inference.** A provider can read the facts of a definition *it constructed*, because it holds it. An out-of-crate reference capability wanting to conform to the governed `f32` arithmetic — the consumer the facts exist for — appears to have no way to obtain the governed definition and read its facts at all.

## What to settle

Either establish that a consumer can reach a governed semantic definition, and name the accessor and what it returns; or establish that it deliberately cannot, and say what a conforming out-of-crate provider is meant to read instead — the published field constants describe records it may have no way to obtain, and a vocabulary for an unreachable record is a documentation defect rather than a boundary.

**Do not add an accessor before answering.** Returning a registered definition is a public surface that hands out a provider's `Arc<dyn OperationInferencer>` and its normative reference, and ADR 0075 reserves that promotion. Check first whether `project_operation_authority` already discharges the need in a narrower shape.

## Closes when

Either a consumer can obtain a governed semantic definition's facts through a named public path, or the contract states that it cannot and names what it reads instead; the fact-field vocabulary's documentation agrees with whichever holds; and `make full` passes.

## Outcome — facts are readable; definitions are not (2026-07-27)

**Fact: `project_operation_authority` does not discharge the need**, which the ticket asked to check first. It returns a `SemanticCapabilityAuthority` carrying three identities — reached definitions, admission provenance, registry snapshot — and every one is an opaque byte string. Those are *comparable* and not *readable*: they tell a consumer whether two authorities agree, never what either says.

**Decision: publish facts, not definitions.** `FrozenSemanticRegistry::operation_facts(&OpKey)` and `value_type_facts(&ValueTypeDefinitionKey)` return the canonical fact records; both are `None` for an unregistered subject.

The narrower shape is the point. A registered definition also carries its provider's `Arc<dyn OperationInferencer>` and its normative reference, and handing those to any holder of the registry promotes a *behaviour* surface. What a conforming out-of-crate capability needs is the record the published fact-field vocabulary interprets — data and nothing else. Note the scalar layer's `FrozenScalarRegistry::definition` already returns the whole definition including its inferencer; that is a precedent this deliberately does **not** follow, because a wider surface existing elsewhere is not a reason to widen this one.

**The gap this closes was real.** `expose-the-governed-fact-field-vocabulary` published field identifiers for records the semantic layer offered no way to obtain, so its test could exercise the scalar constants and not the semantic ones. That test now reads both layers, and the semantic constants are proven at a *read* site rather than only at their construction sites — which is the difference between a fact about the records and an assertion about them.

**Not done, and not needed:** no accessor returns a `ValueTypeDefinition` or an `OperationDefinition`. If a consumer is ever found that needs the normative reference or the inferencer, that is a separate promotion with its own argument; nothing today does.
