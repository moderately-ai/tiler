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

The `research-readiness-gate` ticket is `awaiting-decision`. Its remaining
product choice is whether the first Metal value proof should pull forward the
already-specified strict serial `f32` sum baseline or remain reduction-free.
This status does not authorize implementation.

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
