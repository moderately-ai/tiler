# 0029: Generalize affine quantization granularity with parameter index maps

**Status:** accepted

## Context

Affine quantization may use one parameter pair for an entire tensor, one pair
per axis coordinate, or one pair per multidimensional block. Modeling scale and
zero point as scalar fields makes per-axis and per-block support a later
structural rewrite. Requiring every backend to implement every granularity
before the semantic model admits them would instead couple independent
capabilities.

Quantization parameters select values according to tensor coordinates. This is
an index relationship, but it is semantic rather than a physical storage
addressing decision.

## Decision

Affine quantization granularity is represented by a bounded canonical mapping
from a data-tensor coordinate to the corresponding coordinate in scale and
optional zero-point parameter tensors.

Per-tensor, per-axis, and per-block quantization are built-in forms over this
common concept:

```text
data coordinate [i, j] -> parameter coordinate

per-tensor: []
per-axis:   [j]
per-block:  [floor_div(i, block_i), floor_div(j, block_j)]
```

The exact mapping IR is deferred. It must be typed, bounded, canonical,
shape-checkable, and restricted to admitted semantics rather than arbitrary
user code. Built-in constructors provide the ordinary forms without requiring
users to author index expressions directly.

Representation, structural verification, reference evaluation, optimization,
storage realization, and backend lowering are separate capabilities. An
initial backend may vertically support a subset while the semantic graph
recognizes and verifies the broader built-in granularity model. Unsupported
capabilities fail explicitly.

New numerical scheme families, such as codebook quantization, extend the
versioned numeric-interpretation boundary. They are not encoded as special
cases in the affine parameter map.

## Consequences

- Adding per-axis or per-block lowering does not require changing the logical
  quantization contract.
- Parameter tensor shapes and mappings become explicit verifier obligations.
- View and reindex operations must preserve, transform, or reject the mapping;
  they may not silently detach or reinterpret it.
- Backend support can grow incrementally without confusing representability
  with executability.
- Physical bit packing, byte order, padding, and alignment remain separate
  storage-encoding facts.

## Alternatives considered

A scalar scale/zero-point field is simple for per-tensor quantization but blocks
general granularity. Unrelated per-tensor, per-axis, and per-block variants
duplicate verification and transformation logic. Arbitrary user-defined index
code is maximally extensible but weakens canonical identity, validation,
termination, portability, and explainability. Requiring complete backend
support before representation repeats the coupling rejected by ADR 0026.
