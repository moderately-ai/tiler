---
id: decide-per-dtype-dispatchability-as-a-target-capability
title: Decide whether per-dtype dispatchability is a stated target capability
status: todo
priority: p2
dependencies: []
related: [measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, carry-the-dtype-on-the-metal-subnormal-flush-fact]
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: [research, metal, target-profiles, feasibility]
---
Finding 26 of [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md) measures a case the physical contracts have no name for: the iOS Simulator compiles every `bfloat` probe kernel to LLVM IR, to AIR, and links it to a metallib without a diagnostic, and its device then fails `newComputePipelineStateWithFunction:` with `XPC_ERROR_CONNECTION_INTERRUPTED` — on both the offline and runtime compilation paths. The same simulator runs every `f32` and `f16` kernel in the same invocation. The refusal is measured to be about the *format* rather than any operation on it: the arithmetic-free `materialize_bf16`, which emits zero floating-point operations, is refused too.

So "it compiled for this target, therefore it runs there" is false on a measured row, and the dtype is what makes it false.

**User-visible outcome.** A program using a dtype that the selected device
cannot dispatch must be rejected at the earliest point where Tiler can know
that fact, with an explanation that identifies the dtype and target. It must
never be packaged as though it were executable, reach the one-way routing
commit, and then silently fall back.

The work is to determine which component has enough reliable evidence to make
that rejection. Eliminate any candidate that cannot preserve that outcome
before asking for an architectural choice:

- **Profile-owned capability evidence.** A planner can reject a `bfloat` plan
  before producing an artifact, but only if the capability is scoped by the
  target family and other authorities that can change the answer. An unmeasured
  entry must reject rather than default.
- **Device preflight evidence.** Pipeline creation can test the actual device,
  but the result arrives after artifact production. Establish whether that
  check can occur before routing commit and program work and whether the
  artifact must disclose that it still requires the check.

The failure is loud rather than silent today, so this is not evidence of a
wrong result. It is a placement and explainability gap.

**What this is not.** Not the cause of the simulator's refusal, which is unmeasured and is not on the critical path for the decision — whether *this* runtime lacks `bfloat` lowering does not change whether the architecture should carry per-dtype dispatchability. Not the subnormal-flush fact's dtype, which [carry-the-dtype-on-the-metal-subnormal-flush-fact](carry-the-dtype-on-the-metal-subnormal-flush-fact.md) holds.

**What closes this.** The reliable evidence source and earliest rejection point
are established; candidates that cannot meet the user-visible outcome are
discarded with their derivation; and the surviving contract is recorded, or a
genuine remaining product choice is presented atomically with a recommendation
and counterpoint.
