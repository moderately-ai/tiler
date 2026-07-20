---
schema: "tiler-doc/v1"
id: "tiler.portal.status"
kind: "portal"
title: "Project status"
topics: ["status", "orientation"]
related: ["tiler.questions.open", "tiler.roadmap"]
---

# Project status

Tiler has entered a bounded prototype phase. The semantic,
optimizer, scheduling, numerical, artifact, cache, and runtime boundaries have
substantial accepted decisions and bounded executable evidence. Production
compiler crates and kernels have not been implemented or stabilized.

## Authorized prototype

Tom selected and authorized the bounded strict serial `f32` `Sum` Metal value
proof in [ADR 0055](decisions/0055-use-a-serial-sum-for-the-first-metal-value-proof.md).
The prototype must pass through the documented semantic, reference, optimizer,
schedule, structured-kernel, artifact, and guarded-runtime boundaries; a
handwritten standalone Metal kernel is insufficient. Crate layout and MSRV are
the next decisions before scaffolding.

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
