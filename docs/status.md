---
schema: "tiler-doc/v1"
id: "tiler.portal.status"
kind: "portal"
title: "Project status"
topics: ["status", "orientation"]
related: ["tiler.questions.open", "tiler.roadmap"]
---

# Project status

Tiler is in the research-to-implementation readiness phase. The semantic,
optimizer, scheduling, numerical, artifact, cache, and runtime boundaries have
substantial accepted decisions and bounded executable evidence. Production
compiler crates and kernels have not been authorized or implemented.

## Current gate

The [`research-readiness-gate`](../tickets/research-readiness-gate.md) ticket is
`awaiting-decision`. Its two remaining product choices are deliberately atomic
and sequential: [Q-PLAN-017](open-questions.md#q-plan-017--first-metal-value-proof-workload)
selects the proposed strict serial `f32` sum profile or reduction-free plumbing;
then [Q-PHASE-001](open-questions.md#q-phase-001--implementation-phase-authorization)
authorizes, narrows, or declines that implementation phase.

If Tom authorizes implementation, crate layout and MSRV remain deliberately
sequenced follow-up decisions before scaffolding; the gate's closure must create
their tickets and the chosen vertical-slice tickets. No implementation ticket
is pre-authorized while the gate is parked.

## Evidence boundary

- Executable spikes validate specific representations, protocols, numerical
  cases, and toolchain behavior.
- Apple compatibility is bounded to measured toolchains and hosts; old-OS and
  device-family runtime coverage is not universal.
- Sound region-accuracy analysis is feasible for a narrow trusted-analyzer
  profile; independent certificate checking remains deferred.
- Multi-device, external-storage, and richer transfer work is deliberately
  deferred, not silently ready.

## Live work state

Ticketsplease is authoritative for current workflow state:

```sh
tkt rollup
tkt ready
tkt tracks
tkt reconcile
```

See the [work-tracking guide](work-tracking.md) before claiming work. The
[roadmap](roadmap.md) describes proposed progression, not completed support.
