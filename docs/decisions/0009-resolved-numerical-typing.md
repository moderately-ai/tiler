# 0009: Resolve numerical typing before semantic optimization

**Status:** accepted

## Context

Tensor frontends disagree about implicit promotion, weak scalar types, default
dtypes, reduction widening, and autocast. GPU operations also distinguish
input storage/value type, internal computation precision, accumulator type,
result type, rounding, and algorithm. Leaving any of these decisions to an
ambient frontend or backend default would make semantic rewrites, fusion, and
fallback depend on state absent from canonical program identity.

## Decision

Every tensor value admitted to compilable semantic IR has a resolved value
dtype. Every operation has a resolved numerical signature.

Ordinary elementwise operations are homogeneous by default. A frontend may
apply its own versioned promotion, weak-scalar, default-dtype, or autocast
policy, but it emits explicit typed constants and semantic conversion
operations before canonical admission. Later compiler phases never reapply the
frontend's ambient policy.

Reductions, contractions, and other intrinsically mixed-precision operations
use specialized semantic signatures. Where applicable, a signature explicitly
defines per-operand computation/input precision, accumulator dtype, result
value dtype, conversion behavior, and reduction-order, contraction, or
algorithm requirements. These roles do not collapse into one generic `dtype`
field.

A semantic cast or quantization boundary remains observable when fusion removes
physical materialization. Physical storage representation and allocation are
separate decisions and cannot introduce or erase semantic rounding.

Backend feasibility classifies a resolved contract as exact native support,
exact emulation, support only under an already permitted relaxation, or
unsupported. Backend defaults never silently widen the program's numerical
permissions.

## Consequences

- Canonical graph identity, reference evaluation, fallback, and artifact
  identity agree on all resolved numerical types and conversion points.
- Frontends retain ergonomic promotion and autocast policies without making
  compiler core depend on them.
- Operation extensions must define or resolve their numerical signature before
  they become optimizable.
- Reductions and contractions need richer schemas and conformance tests than
  ordinary elementwise operations.
- Exact fusion preserves value-level rounding even when it removes memory
  traffic.

## Alternatives considered

A universal implicit promotion lattice would privilege one frontend and still
fail to specify reductions and contractions. Preserving unresolved promotion
until lowering would make backend selection part of tensor meaning. Requiring
graph-level casts for every scalar step inside a reduction would be explicit
but unnecessarily encode an operation's internal scalar iteration in the
public tensor graph.
