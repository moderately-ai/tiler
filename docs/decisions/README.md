# Architecture decision records

ADRs record choices that constrain several components or would be expensive to
reverse. Proposed ADRs and design text remain non-decisions until explicitly
accepted. Unresolved questions are collected in
[open questions](../open-questions.md).

## Index

- [0001: Separate semantic planning from physical scheduling](0001-separate-semantic-and-physical-plans.md) — proposed
- [0002: Generate Metal artifacts ahead of time](0002-aot-metal-artifacts.md) — proposed
- [0003: Keep the compiler independent of Candle](0003-candle-is-an-integration.md) — proposed
- [0004: Treat each inline macro invocation as an AOT bundle](0004-inline-macro-aot-bundles.md) — proposed
- [0005: Expose a public semantic graph and extension boundary](0005-public-semantic-tensor-graph.md) — accepted
- [0006: Model semantic programs as operation/value graphs](0006-operation-value-graph.md) — proposed
- [0007: Make normalized hardware schedules first-class IR](0007-first-class-kernel-schedules.md) — proposed
- [0008: Separate extent symbols from typed root bindings](0008-typed-root-bindings.md) — accepted
- [0009: Resolve numerical typing before semantic optimization](0009-resolved-numerical-typing.md) — accepted
- [0010: Make conversion behavior a typed semantic contract](0010-typed-conversion-contracts.md) — accepted
- [0011: Resolve numerical permissions per operation](0011-per-operation-numerical-permissions.md) — accepted
- [0012: Keep reduction topology in physical plans](0012-physical-reduction-topology.md) — accepted
- [0013: Scope deterministic numerical guarantees](0013-scoped-determinism.md) — accepted
- [0014: Separate reassociation from operand permutation](0014-reassociation-vs-permutation.md) — accepted
- [0015: Distinguish required FMA from optional contraction](0015-fma-vs-contraction.md) — accepted
- [0016: Resolve transcendental accuracy per operation](0016-transcendental-accuracy-contracts.md) — accepted
- [0017: Separate local semantics from region accuracy goals](0017-local-vs-region-accuracy.md) — accepted

## Template

```markdown
# NNNN: Decision title

**Status:** proposed | accepted | superseded

## Context

## Decision

## Consequences

## Alternatives considered
```
