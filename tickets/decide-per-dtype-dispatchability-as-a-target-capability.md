---
id: decide-per-dtype-dispatchability-as-a-target-capability
title: Decide whether per-dtype dispatchability is a stated target capability
status: in-progress
priority: p2
dependencies: []
related: [measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, carry-the-dtype-on-the-metal-subnormal-flush-fact]
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: [research, metal, target-profiles, feasibility]
claimed_from: todo
assignee: agent-dispatchability
lease_expires_at: 1785042783
---
Finding 26 of [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md) measures a case the physical contracts have no name for: the iOS Simulator compiles every `bfloat` probe kernel to LLVM IR, to AIR, and links it to a metallib without a diagnostic, and its device then fails `newComputePipelineStateWithFunction:` with `XPC_ERROR_CONNECTION_INTERRUPTED` — on both the offline and runtime compilation paths, and for the arithmetic-free kernel as well as the arithmetic ones. The same simulator runs every `f32` and `f16` kernel in the same invocation.

So "it compiled for this target, therefore it runs there" is false on a measured row, and the dtype is what makes it false.

**The decision.** Whether per-dtype dispatchability becomes a stated target capability with a typed feasibility rejection before plan selection, or stays a runtime preflight failure. Both are defensible and they encode different priorities, which is why this is a decision rather than a research question:

- **As a target capability.** A planner rejects a `bfloat` plan for a family whose profile does not declare it, with an explainable reason, before any artifact is produced. Costs a capability axis that has to be populated per family and per toolchain row, and an unmeasured entry has to reject rather than default — which is the same discipline finding 24 forces on the subnormal-flush fact.
- **As a runtime preflight failure.** Nothing is declared; the failure arrives at pipeline creation and is loud. Costs nothing up front and wastes the compile and the artifact, and it arrives *after* artifact production rather than before plan selection — which ADR 0051 already constrains: a fallback is legal only before program work.

The failure is loud rather than silent today, which is the good direction, so this is not a correctness defect being deferred. It is a question of where the rejection belongs.

**What this is not.** Not the cause of the simulator's refusal, which is unmeasured and is not on the critical path for the decision — whether *this* runtime lacks `bfloat` lowering does not change whether the architecture should carry per-dtype dispatchability. Not the subnormal-flush fact's dtype, which [carry-the-dtype-on-the-metal-subnormal-flush-fact](carry-the-dtype-on-the-metal-subnormal-flush-fact.md) holds.

**What closes this.** An accepted decision recorded in the relevant contract, or an explicit deferral naming the trigger that would reopen it.
