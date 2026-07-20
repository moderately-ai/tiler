# 0001: Separate semantic planning from physical scheduling

**Status:** accepted

## Context

Tensor meaning and GPU execution have different equivalence rules and rates of
change. Reindexing, broadcasting, mapping, and reduction semantics should not
depend on Metal lane mappings, threadgroup sizes, or local-memory algorithms.

## Decision

Tiler will represent backend-neutral, explicitly environment-parameterized
semantic tensor plans separately from target-aware physical schedules. A
global optimizer selects fusion and materialization alternatives; a local
scheduler maps each candidate region to hardware. Scheduled programs lower
into a typed structured kernel IR.

## Consequences

- Frontends can share tensor semantics across backends.
- Numerical and fusion legality are testable before code generation.
- Several physical schedules can implement one semantic group.
- Additional representations and verifiers are required.
- Cross-layer shortcuts must be resisted during early prototyping.

## Alternatives considered

A single GPU-oriented IR is initially smaller but makes target decisions part
of tensor meaning and encourages backend compilation to become validation.
