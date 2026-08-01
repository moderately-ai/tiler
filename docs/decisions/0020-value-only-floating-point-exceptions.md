---
schema: "tiler-doc/v1"
id: "ADR-0020"
kind: "decision"
title: "Use value-only floating-point exceptions initially"
topics: ["numerics","floating-point","exceptions"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
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

## Implementation boundary

Added 2026-08-01 by [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](../../tickets/re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which moved `implementation_status` from `not-started` to `partial`. This section states which clauses that value rests on and which it does not, read at `2aa0824`. It is a status record and adds no decision.

**Realized — every operation declares its effect class, and that declaration enters durable identity.** `OperationEffect` (`crates/tiler-ir/src/semantic/operation.rs:988`) is a required field of every `OperationDefinition`, encoded into the definition's canonical identity at `crates/tiler-ir/src/semantic/registry.rs:2544` through an exhaustive match, and every registered family declares `Pure`. It is deliberately not `#[non_exhaustive]` because three encoders outside `tiler-ir` map the vocabulary totally, so a second effect class is a build error at each of them rather than a silent re-encoding — which is this decision's "rejected rather than silently interpreted as value-only behavior", enforced by the compiler at the IR layer.

**Realized — exceptional cases produce the values their resolved contracts define.** `ExceptionalValueContract` (`crates/tiler-ir/src/semantic/accuracy/contract.rs:227`) states four rules explicitly and takes four required arguments "because a defaulted exceptional rule is a behaviour nobody chose": the NaN reference, the infinite reference, an input outside the admitted domain, and a finite reference above the format's range. Real operations carry resolved instances — the softmax's subordinate exponential (`crates/tiler-ir/src/semantic/softmax.rs:499`), the activation's (`crates/tiler-ir/src/semantic/silu.rs:291`), and the normalization's reciprocal square root (`crates/tiler-ir/src/semantic/rms_norm.rs:397`) — and the arithmetic families separately declare their canonical NaN payload and the boundary it is applied at. Nothing anywhere reads or writes an ambient status flag, a sticky bit, or a trap.

**Realized — the value-only contract is what rewrite legality rests on.** `docs/compiler/optimizer.md` states that pushing a view through a pointwise expression is unobservable because initial floating-point operations are value-only under this decision, so the property is load-bearing for a landed rewrite rather than merely declared.

**Unrealized — the effect vocabulary can resolve only one way.** `OperationEffect` has exactly one variant. The decision fixes the initial contract at value-only, so a single inhabitant is the decided state rather than an unresolvable enum — but a reader should not read `partial` as evidence that an alternative exception-observation mode is representable, negotiable, or rejectable at a version boundary. There is no named `RaiseNoFlag`-style contract value, and no operation states which exception-observation model it uses.

**Unrealized — the deferred half has no representation at all.** No versioned effect signature, resource or effect-token value kind, ordering or liveness rule, or partial-execution contract exists, and no schema or artifact field carries an effect model, so "schema and artifact versioning must fail closed for unknown effect models" has nothing to fail closed on. `docs/roadmap.md` records the same boundary and names the trigger that would open it.

**Unrealized — the pure multi-result diagnostic operation.** No registered operation exposes diagnostics as ordinary tensor data through additional results.

## Alternatives considered

Treating exception behavior as unspecified would allow backend defaults to
change semantics and would make future compatibility ambiguous. Modeling flags
and traps immediately would expand the initial graph, optimizer, runtime, and
fallback contracts before a tensor-kernel use case requires them. Permanently
forbidding effectful floating-point behavior would unnecessarily constrain the
toolkit's future scope.
