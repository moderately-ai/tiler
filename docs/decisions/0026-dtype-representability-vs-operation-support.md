---
schema: "tiler-doc/v1"
id: "ADR-0026"
kind: "decision"
title: "Separate dtype representability from operation support"
topics: ["numerics","dtypes","capabilities"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.mature-dtype-taxonomy"]
ticket: "enumerate-the-mature-tensor-dtype-taxonomy"
---

# 0026: Separate dtype representability from operation support

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [mature dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md).
- **Work record:** [enumerate-the-mature-tensor-dtype-taxonomy](../../tickets/enumerate-the-mature-tensor-dtype-taxonomy.md).


## Context

Modern tensor systems recognize element formats whose operation and backend
coverage is incomplete. An FP8 value may be valid storage and a matrix operand
while lacking general scalar arithmetic. A packed integer may support copying
and reshaping without supporting element-addressable views. Requiring complete
operation coverage before a dtype can enter semantic IR makes type evolution
unnecessarily monolithic.

Conversely, treating dtype presence as universal support would admit malformed
programs and produce late backend failures.

## Decision

Canonical semantic IR may represent a recognized, exact element type before
every operation, evaluator, optimizer pass, or backend supports it.

Each operation invocation is verified against its complete typed signature and
declared semantic capabilities. A representable tensor may participate only in
operations that explicitly admit that dtype/signature. For example, a
bit-preserving reindex may support a type for which `Exp` has no semantics.

Representable does not mean unknown. Every admitted nominal type has a stable,
versioned canonical identity and registered definition; initial verified graphs
reject unknown type identities. Backend planning separately proves storage
encoding, ABI, memory-space, target, and operation realization.

Support is therefore a capability lattice including recognition,
representability, literals, operation semantics, reference evaluation,
conversion, optimization, storage/layout, ABI/interchange, lowering, native or
emulated realization, and product-profile enablement.

## Consequences

- New dtypes can enter storage/view or specialized-operation paths without a
  fictitious complete arithmetic surface.
- Errors identify the unsupported operation/type capability rather than calling
  the dtype globally unsupported.
- Frontends and extensions can preserve exact type identity through compatible
  regions.
- Reference evaluation and optimization remain mandatory only for operations
  claiming those capabilities.
- Product profiles can remain vertically constrained while the canonical type
  universe evolves.

## Alternatives considered

Admitting only fully executable types gives a simple Boolean support model but
couples unrelated operations and targets. Making every known bit pattern an
unchecked opaque type preserves data but defeats semantic verification.
Inferring operation support from backend storage support confuses
representability with executable semantics.
