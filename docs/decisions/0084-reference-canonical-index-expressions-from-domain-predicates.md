---
schema: "tiler-doc/v1"
id: "ADR-0084"
kind: "decision"
title: "Reference canonical index expressions from domain predicates"
topics: ["indexing", "predicates", "proof", "ir"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.constraint-prover-boundary", "tiler.research.indexing.index-access-model"]
depends_on: ["ADR-0046", "ADR-0061", "ADR-0074"]
ticket: "implement-index-domain-predicates"
---

# 0084: Reference canonical index expressions from domain predicates

**Status:** accepted. Tom authorized the semantic design and ratified the exact public Rust types on 2026-07-28.

## Context

The index verifier can prove many coordinate bounds structurally, by interval propagation, or by bounded finite enumeration. When those lanes establish neither a proposition nor its negation, the proposition currently has no durable typed representation. A proof-resource stop is retained only as a diagnostic, and a wholly unbounded symbolic case is rejected.

ADR 0046 already admits affine, constant-divisor quasi-affine, and guarded semi-affine index expressions. Defining a second affine or Presburger expression tree for residual predicates would create another semantic, validation, and identity authority. It would also fail to state a bound such as `i floordiv d < M` when `d` is a proven-positive symbolic divisor.

The shape-prover boundary already fixes three proof outcomes: `Proved`, `Disproved`, and `Unknown` with a structured reason. A prover limitation or resource stop is therefore not another evidence class and does not justify treating a proposition as true or false.

## Decision

An index-domain predicate is one atom in a closed typed vocabulary. A verified region carries atoms as an implicit conjunction. Each atom references canonical verified index-expression nodes and region-owned sourced extents; it never embeds another expression tree.

The initial atoms are:

```rust,ignore
enum IndexExtentRef {
    Dimension(VerifiedDimensionId),
    TensorAxis {
        tensor: VerifiedTensorId,
        axis: u32,
    },
}

enum IndexDomainPredicate {
    NonNegative {
        expression: VerifiedIndexExprId,
    },
    LessThanExtent {
        expression: VerifiedIndexExprId,
        extent: IndexExtentRef,
    },
}
```

Together these state the two coordinate-bound obligations for every admitted expression class: `0 <= e` and `e < extent`. The extent selector points at the region entity that owns the static or symbolic extent, so it neither copies an extent value nor publishes a second constant-or-symbol vocabulary.

Both enums are deliberately exhaustive rather than `#[non_exhaustive]`. Identity encoders, proof engines, explain renderers, and semantic-discharge stages must reconsider a newly added atom or extent source at compile time. The types are leaf descriptors; constructing one does not attach it to a region or establish that its handles are valid. Only the checked region lifecycle may retain one as a verified obligation.

The predicate vocabulary contains no arbitrary Boolean composition, provider-defined atom, physical guard, runtime check, or proof result. A later semantic-discharge stage may erase a proved obligation, retain an explicit `Unknown`, or realize a named semantic host check before program work. It may not silently turn an unproved predicate into a physical variant guard.

Proof exchange retains the accepted three outcomes. `Unknown(InsufficientFacts)`, `Unknown(UnsupportedFragment)`, and `Unknown(ResourceLimit)` remain reasons why neither entailment was established, not evidence and not a fifth outcome.

## Consequences

- Predicate identity reuses canonical region entities and adds only the atom kind and referenced canonical subjects.
- Any present or future semi-affine expression is stateable without widening the predicate language merely because the expression language grew.
- Bounds predicates remain separate from write-ownership obligations, which quantify over the access relation rather than one coordinate expression.
- The current constant-divisor IR can adopt the vocabulary immediately. The symbolic-divisor example becomes constructible only after `represent-semi-affine-index-expressions-in-the-ir` lands; this decision does not pretend that representation already exists.
- Provers may conservatively return `Unknown` for semi-affine fragments they cannot analyze. A verified executable program still requires a named semantic discharge before work begins.

## Alternatives considered

A closed affine-inequality AST is sound but duplicates the canonical index vocabulary and cannot state symbolic-divisor obligations. It would require a public identity change when widened.

A quantifier-free Presburger AST adds Boolean and case machinery without a demonstrated index-bound obligation requiring it, duplicates expression authority, and still cannot state symbolic division. Its decision procedure also needs a resource limit, so it does not eliminate `Unknown`.

Encoding arbitrary expressions or callbacks inside predicates would make validation, canonical identity, reference semantics, and bounded proof exchange open-ended. Encoding a runtime or physical guard would collapse semantic validity into implementation applicability, contrary to ADR 0046.
