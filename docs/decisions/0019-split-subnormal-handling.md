# 0019: Separate subnormal input and result handling

**Status:** accepted

## Context

Treating a subnormal operand as zero before an operation and flushing a newly
produced subnormal result to zero are observably different behaviors. Some
targets couple them in one execution mode, while others expose or require
different combinations. A single `flush_subnormals` boolean cannot state which
behavior occurred.

## Decision

Every applicable floating-point operation resolves subnormal input handling
and subnormal result handling independently. Each dimension initially supports
preservation or an explicit flush-to-zero behavior; zero-sign behavior is
resolved with the signed-zero contract.

Portable-bitwise execution preserves both input and result subnormals. Relaxed
operation contracts may permit either or both kinds of flushing. A backend that
cannot realize a requested combination natively must emulate it, consume an
already authorized relaxation, or reject the plan.

Backend switches that couple input and result flushing do not couple Tiler's
semantic permissions.

## Consequences

- Reference evaluation can distinguish input flushing from result flushing.
- Backend feasibility accurately represents partially supported combinations.
- Portable-bitwise results retain gradual underflow.
- Relaxed modes can match useful hardware behavior without silently changing
  both sides of an operation.
- Subnormal and signed-zero policy both participate in artifact identity and
  adversarial tests.

## Alternatives considered

One flush-to-zero flag is compact but loses observable information. Treating
all subnormal behavior as backend-defined makes fusion and fallback disagree.
Requiring preservation in every conformance mode unnecessarily excludes
explicitly requested fast execution.
