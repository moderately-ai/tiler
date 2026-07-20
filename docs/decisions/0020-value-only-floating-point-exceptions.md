---
schema: "tiler-doc/v1"
id: "ADR-0020"
kind: "decision"
title: "Use value-only floating-point exceptions initially"
topics: ["numerics","floating-point","exceptions"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.operation-conformance-matrix"]
ticket: "numerical-policy-contract"
---

# 0020: Use value-only floating-point exceptions initially

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [operation conformance matrix](../research/numerics/operation-conformance-matrix.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Floating-point exceptions can mean either exceptional result values, such as
NaN or infinity, or observation of a mutable floating-point environment through
sticky flags and traps. The first fits a pure tensor dataflow graph. The second
introduces effects, ordering, liveness, and partial-execution behavior.

StableHLO specifies value-producing, no-status-flag behavior. LLVM likewise
requires constrained floating-point operations when exception behavior is
observable, and CUDA documents no per-thread floating-point status register or
trap handlers. These precedents support a value-only initial contract, but do
not justify making future effectful semantics impossible.

## Decision

Initial Tiler floating-point operations use an explicit value-only,
`RaiseNoFlag`-style exception-observation contract. Exceptional cases produce
the values defined by their resolved operation contracts; they do not expose
ambient status flags or synchronous traps.

An operation may expose diagnostics as ordinary tensor data, including through
multiple results. Such an operation remains pure.

True floating-point-environment observation or mutation is deferred. It may be
added only through new versioned effect signatures and explicit
resource/effect-token value kinds, with corresponding verifier, optimizer,
runtime, ABI, artifact, and partial-execution contracts. Existing tensor value
kinds and pure operation identities keep their meaning. Unsupported future
exception modes, effect signatures, and value kinds are rejected rather than
silently interpreted as value-only behavior.

## Consequences

- Initial semantic graphs remain pure tensor SSA and need no hidden ordering
  edges for floating-point flags.
- Reference evaluation and backend conformance test exceptional result values,
  not ambient processor or device flags.
- A pure multi-result diagnostic operation can be added without first designing
  a general effect system.
- Future traps, sticky flags, or ordered clear/read operations remain possible,
  but require an explicit effect-model expansion rather than an attribute on a
  nominally pure tensor node.
- Schema and artifact versioning must fail closed for unknown effect models.

## Alternatives considered

Treating exception behavior as unspecified would allow backend defaults to
change semantics and would make future compatibility ambiguous. Modeling flags
and traps immediately would expand the initial graph, optimizer, runtime, and
fallback contracts before a tensor-kernel use case requires them. Permanently
forbidding effectful floating-point behavior would unnecessarily constrain the
toolkit's future scope.
